use bevy::prelude::*;
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientId, ClientMessageAppExt, FromClient, Replicated, ServerState,
};
use bevy_replicon::shared::backend::connected_client::NetworkId as RepliconNetworkId;
use unboard_core::components::spawning::PlayerSpawnPoint;
use uninteraction_core::interaction::ExecuteInteractionEvent;
use unplayer_core::components::{Hiding, MainPlayer, PlayerSpectating, PlayerSprite, Stamina};
use unreplicon_core::components::{LobbyInfo, RepliconPlayerSpawningActive};
use unreplicon_core::messages::{InteractionRequestMessage, PlayerMoveMessage};
use unreplicon_core::net_components::{
    EMFMeterNet, FlashlightNet, NetworkPosition, PlayerNetInfo, PlayerStateNet, RepellentFlaskNet,
    SageBundleNet, SpiritBoxNet, ThermometerNet,
};
use unspatial_core::boardposition::{BoardPosition, MapEntityFieldBPos};
use unspatial_core::position::Position;
use untypes_core::states::AppState;

pub(super) fn app_setup(app: &mut App) {
    // Register client → server messages
    app.add_client_message::<PlayerMoveMessage>(Channel::Unreliable);
    app.add_client_message::<InteractionRequestMessage>(Channel::Ordered);

    // Register replicated components
    app.replicate::<NetworkPosition>();
    app.replicate::<PlayerNetInfo>();
    app.replicate::<PlayerStateNet>();
    app.replicate::<FlashlightNet>();
    app.replicate::<ThermometerNet>();
    app.replicate::<EMFMeterNet>();
    app.replicate::<SpiritBoxNet>();
    app.replicate::<SageBundleNet>();
    app.replicate::<RepellentFlaskNet>();

    // Server-side: spawn/tag player entities when InGame starts
    app.add_systems(
        OnEnter(AppState::InGame),
        setup_mission_players.run_if(in_state(ServerState::Running)),
    );

    // Server-side: message handlers + host state sync
    app.add_systems(
        Update,
        (
            handle_player_move,
            handle_interaction_request,
            sync_player_state_to_net,
        )
            .run_if(in_state(ServerState::Running)),
    );

    // Client-side: send local position to server (Join clients only)
    app.add_systems(
        Update,
        send_local_player_position
            .run_if(in_state(AppState::InGame))
            .run_if(not(in_state(ServerState::Running))),
    );

    // Both server and client: interpolate remote player positions in InGame
    app.add_systems(
        Update,
        interpolate_remote_players.run_if(in_state(AppState::InGame)),
    );

    // Cleanup the spawning-active marker when leaving InGame
    app.add_systems(OnExit(AppState::InGame), cleanup_mission_players);
}

/// Helper: resolve a `ClientId` to the u64 replicon NetworkId.
///
/// `ClientId::Server` → 0 (our sentinel for the host / listen-server player).
fn client_network_id(client_id: ClientId, q_network_id: &Query<Option<&RepliconNetworkId>>) -> u64 {
    match client_id {
        ClientId::Server => 0,
        ClientId::Client(entity) => q_network_id
            .get(entity)
            .ok()
            .flatten()
            .map(|n| n.get())
            .unwrap_or(0),
    }
}

/// Server: called once on `OnEnter(AppState::InGame)`.
///
/// 1. Inserts `RepliconPlayerSpawningActive` so that `spawn_joined_player` in
///    `unclassic-mode-plugin` is suppressed.
/// 2. Adds `(Replicated, NetworkPosition, PlayerStateNet, PlayerNetInfo)` to the
///    host's player entity (which the orchestrator already spawned with full visuals).
/// 3. Spawns bare replicated entities for every remote client found in `LobbyInfo`
///    so that connected clients receive them and can add their own visuals.
fn setup_mission_players(
    q_host_player: Query<(Entity, &Position, &PlayerSprite), Without<NetworkPosition>>,
    q_lobby: Query<&LobbyInfo>,
    q_spawn_points: Query<&Position, (With<PlayerSpawnPoint>, Without<PlayerSprite>)>,
    mut commands: Commands,
) {
    // Signal that replicon-based player spawning is now active.
    commands.insert_resource(RepliconPlayerSpawningActive);

    let spawn_points: Vec<Position> = q_spawn_points.iter().copied().collect();
    let default_pos = Position {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        visual_priority: 0.0,
    };

    // Add network components to the existing host player entity.
    for (entity, pos, _player_sprite) in q_host_player.iter() {
        // Look up the host's tint from LobbyInfo (client_id == 0 is the host).
        let tint_color_index = q_lobby
            .iter()
            .flat_map(|l| l.players.iter())
            .find(|p| p.client_id == 0)
            .map(|p| p.tint_color_index)
            .unwrap_or(0);

        commands.entity(entity).insert((
            Replicated,
            NetworkPosition {
                x: pos.x,
                y: pos.y,
                z: pos.z,
            },
            PlayerStateNet::default(),
            PlayerNetInfo {
                client_id: 0,
                tint_color_index,
            },
        ));

        info!(
            "setup_mission_players: host player entity {:?} marked Replicated",
            entity
        );
    }

    // Spawn bare network entities for remote clients.
    let Ok(lobby) = q_lobby.single() else {
        warn!("setup_mission_players: no LobbyInfo entity found — skipping remote player spawn");
        return;
    };

    for (idx, player) in lobby.players.iter().enumerate() {
        if player.client_id == 0 {
            // Host handled above.
            continue;
        }

        // Pick a deterministic spawn point based on lobby order.
        let spawn_pos = spawn_points
            .get(idx % spawn_points.len().max(1))
            .copied()
            .unwrap_or(default_pos);

        let remote_entity = commands
            .spawn((
                Replicated,
                NetworkPosition {
                    x: spawn_pos.x,
                    y: spawn_pos.y,
                    z: spawn_pos.z,
                },
                PlayerStateNet::default(),
                PlayerNetInfo {
                    client_id: player.client_id,
                    tint_color_index: player.tint_color_index,
                },
            ))
            .id();

        info!(
            "setup_mission_players: spawned replicated entity {:?} for client {}",
            remote_entity, player.client_id
        );
    }
}

/// Server: on exit from `AppState::InGame`, remove the spawning-active marker.
///
/// This allows subsequent missions (offline or new online sessions) to
/// re-evaluate whether replicon-based spawning is needed.
fn cleanup_mission_players(mut commands: Commands) {
    commands.remove_resource::<RepliconPlayerSpawningActive>();
}

/// Server: sync the host player's local `Position` and `PlayerSprite` into the
/// replicated `NetworkPosition` and `PlayerStateNet` every frame.
///
/// This is the server-side equivalent of `send_local_player_position` for the
/// listen-server host: since the host IS the server, position updates are made
/// directly on the entity rather than going through a network message.
fn sync_player_state_to_net(
    mut q_host: Query<
        (
            &Position,
            &PlayerSprite,
            &Stamina,
            Has<Hiding>,
            Has<PlayerSpectating>,
            &mut NetworkPosition,
            &mut PlayerStateNet,
        ),
        With<MainPlayer>,
    >,
) {
    for (pos, sprite, stamina, is_hiding, is_spectating, mut net_pos, mut state_net) in
        q_host.iter_mut()
    {
        net_pos.x = pos.x;
        net_pos.y = pos.y;
        net_pos.z = pos.z;

        state_net.is_hiding = is_hiding;
        state_net.is_spectating = is_spectating;
        state_net.is_running = stamina.running;
        state_net.stamina = if stamina.max > 0.0 {
            stamina.current / stamina.max
        } else {
            1.0
        };
        state_net.health = sprite.health;
        state_net.sanity = sprite.sanity;
    }
}

/// Server: handle `PlayerMoveMessage` from connected clients.
///
/// Finds the player entity whose `PlayerNetInfo.client_id` matches the sender
/// and updates its `NetworkPosition`/`PlayerStateNet`. Replicon then propagates
/// the changed components to all other clients.
fn handle_player_move(
    mut reader: MessageReader<FromClient<PlayerMoveMessage>>,
    q_network_id: Query<Option<&RepliconNetworkId>>,
    mut q_players: Query<(&PlayerNetInfo, &mut NetworkPosition, &mut PlayerStateNet)>,
) {
    for msg in reader.read() {
        let sender_id = client_network_id(msg.client_id, &q_network_id);

        for (net_info, mut net_pos, mut state_net) in q_players.iter_mut() {
            if net_info.client_id != sender_id {
                continue;
            }

            net_pos.x = msg.x;
            net_pos.y = msg.y;
            net_pos.z = msg.z;

            state_net.is_running = msg.is_running;
            state_net.frame = msg.frame;
            state_net.is_hiding = msg.is_hiding;
            state_net.stamina = msg.stamina;
            state_net.health = msg.health;
            state_net.sanity = msg.sanity;

            break;
        }
    }
}

/// Server: handle `InteractionRequestMessage` from connected clients.
///
/// Looks up the entity at the given board position and fires
/// `ExecuteInteractionEvent` on the server. The resulting component change
/// (e.g., door open/closed) is then replicated to all clients.
fn handle_interaction_request(
    mut reader: MessageReader<FromClient<InteractionRequestMessage>>,
    q_interactive: Query<(Entity, &MapEntityFieldBPos)>,
    mut ev_interact: MessageWriter<ExecuteInteractionEvent>,
) {
    for msg in reader.read() {
        let target_bpos = BoardPosition {
            x: msg.position[0] as i64,
            y: msg.position[1] as i64,
            z: msg.position[2] as i64,
        };

        let found = q_interactive
            .iter()
            .find(|(_, bpos)| bpos.0 == target_bpos)
            .map(|(e, _)| e);

        if let Some(entity) = found {
            ev_interact.write(ExecuteInteractionEvent {
                entity,
                ietype: msg.ietype.clone(),
                force_tuid: msg.force_tuid,
            });
        } else {
            warn!(
                "handle_interaction_request: no entity found at board position {:?}",
                target_bpos
            );
        }
    }
}

/// Client (Join mode only): read the local player's `Position` every frame and
/// send a `PlayerMoveMessage` to the server.
///
/// The server validates movement and writes the updated position into the
/// authoritative `NetworkPosition` for that player's replicated entity, after
/// which all other clients see the movement via replication.
fn send_local_player_position(
    q_local: Query<
        (
            &Position,
            &PlayerSprite,
            &Stamina,
            Has<Hiding>,
            Has<PlayerSpectating>,
        ),
        With<MainPlayer>,
    >,
    mut writer: MessageWriter<PlayerMoveMessage>,
) {
    let Ok((pos, sprite, stamina, is_hiding, is_spectating)) = q_local.single() else {
        return;
    };

    writer.write(PlayerMoveMessage {
        x: pos.x,
        y: pos.y,
        z: pos.z,
        is_running: stamina.running,
        frame: 0,
        is_hiding,
        stamina: if stamina.max > 0.0 {
            stamina.current / stamina.max
        } else {
            1.0
        },
        health: sprite.health,
        sanity: if is_spectating { 0.0 } else { sprite.sanity },
    });
}

/// Interpolate remote player entities' `Position` toward their server-authoritative
/// `NetworkPosition` each frame.
///
/// Only runs for entities that do NOT have `MainPlayer` (i.e., remote players),
/// so the local player's lag-free position is never overwritten.
fn interpolate_remote_players(
    time: Res<Time>,
    mut q_remote: Query<(&NetworkPosition, &mut Position), Without<MainPlayer>>,
) {
    const LERP_SPEED: f32 = 15.0;
    let alpha = (LERP_SPEED * time.delta_secs()).min(1.0);

    for (net_pos, mut pos) in q_remote.iter_mut() {
        pos.x += (net_pos.x - pos.x) * alpha;
        pos.y += (net_pos.y - pos.y) * alpha;
        pos.z += (net_pos.z - pos.z) * alpha;
    }
}
