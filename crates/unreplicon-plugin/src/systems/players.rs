mod replication;

use bevy::prelude::*;
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientId, ClientMessageAppExt, FromClient, Replicated, ServerMessageAppExt,
};
use std::collections::HashMap;
use unboard_core::components::spawning::PlayerSpawnPoint;
use uncommon_states_core::UIContextState;
use unmission_core::types::SimulationState;
use unplayer_core::components::{PlayerSpawnRequest, PlayerSprite};
use unreplicon_core::components::{
    LobbyInfo, LobbyPlayerInfo, RepliconPlayerSpawningActive, SimulationAuthorized,
};
use unreplicon_core::messages::{
    ConnectionTicketMessage, FloorGearDespawnBroadcast, FloorGearSpawnBroadcast,
    RelieveSimulationAuthority, RequestJoinMission, RequestSimulationAuthority,
};
use unreplicon_core::network_id::NetworkId;
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unreplicon_core::resources::AuthorityRole;
use unreplicon_core::resources::LobbyPresenceRole;
use unreplicon_core::resources::{ClientUuidMap, LocalPlayer, LocalPlayerRole, is_pure_client};
use unspatial_core::position::Position;
use uuid::Uuid;

// Periodic diagnostics for player spawn and ownership handoff state.
fn player_spawn_telemetry(
    q: Query<(
        Entity,
        Option<&PlayerSpawnRequest>,
        Option<&PlayerSprite>,
        Option<&Owner>,
        Option<&SimulationAuthorized>,
        Option<&LocallyOwned>,
    )>,
    authority: Option<Res<AuthorityRole>>,
    local_player: Option<Res<LocalPlayerRole>>,
    spawning_active: Option<Res<RepliconPlayerSpawningActive>>,
    state: Option<Res<State<SimulationState>>>,
    mut frames: Local<u32>,
) {
    *frames += 1;
    if !(*frames).is_multiple_of(60) {
        return;
    }

    trace!(
        "player_spawn_telemetry: tick={} auth={} local_player={} spawning_active={} state={:?}",
        *frames,
        authority.is_some(),
        local_player.is_some(),
        spawning_active.is_some(),
        state.map(|s| *s.get())
    );
    for (e, req, spr, own, sim_auth, locown) in q.iter() {
        if req.is_some() || spr.is_some() {
            trace!(
                "player_spawn_telemetry: entity {:?} spawn_request={} player_sprite={} owner={:?} simulation_authorized={} locally_owned={}",
                e,
                req.is_some(),
                spr.is_some(),
                own.map(|o| o.0),
                sim_auth.is_some(),
                locown.is_some()
            );
        }
    }
}

pub(super) fn app_setup(app: &mut App) {
    // Register client → server messages
    app.add_client_message::<ConnectionTicketMessage>(Channel::Ordered);
    app.add_client_message::<RequestJoinMission>(Channel::Ordered);
    app.add_mapped_client_message::<RequestSimulationAuthority>(Channel::Ordered);
    app.add_mapped_client_message::<RelieveSimulationAuthority>(Channel::Ordered);
    // Register server → client messages
    app.add_server_message::<FloorGearSpawnBroadcast>(Channel::Ordered);
    app.add_server_message::<FloorGearDespawnBroadcast>(Channel::Ordered);
    app.replicate::<SimulationAuthorized>();

    replication::app_setup(app);

    // Periodic diagnostics for player spawning and ownership handoff.
    app.add_systems(Update, player_spawn_telemetry);

    app.add_systems(
        OnEnter(SimulationState::Spawning),
        (
            activate_player_spawning.run_if(resource_exists::<AuthorityRole>),
            auto_request_join_mission,
        ),
    );

    app.add_systems(
        Update,
        (
            handle_request_join_mission,
            handle_simulation_authority_requests,
            handle_simulation_authority_release,
        )
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(SimulationState::Spawning).or(in_state(SimulationState::Ready)))
            .run_if(resource_exists::<RepliconPlayerSpawningActive>),
    );

    app.add_systems(
        Update,
        client_avatar_reconciliation_loop.run_if(is_pure_client),
    );

    // Cleanup the spawning-active marker when leaving InGame.
    app.add_systems(
        OnEnter(SimulationState::TearingDown),
        cleanup_player_spawning_flag,
    );
}

fn to_owner_id(client_id: ClientId) -> OwnerId {
    match client_id {
        ClientId::Server => OwnerId::Server,
        ClientId::Client(entity) => OwnerId::Client(entity),
    }
}

fn throttle_ready(
    entity: Entity,
    elapsed_secs: f32,
    last_sent_at: &mut Local<HashMap<Entity, f32>>,
) -> bool {
    let last = last_sent_at.entry(entity).or_insert(f32::NEG_INFINITY);
    if elapsed_secs - *last >= 0.5 {
        *last = elapsed_secs;
        true
    } else {
        false
    }
}

fn activate_player_spawning(mut commands: Commands) {
    commands.insert_resource(RepliconPlayerSpawningActive);
}

fn auto_request_join_mission(mut ev_join: MessageWriter<RequestJoinMission>) {
    info!("auto_request_join_mission: local map reached Spawning phase -> sending intent-to-play");
    ev_join.write(RequestJoinMission);
}

fn spawn_position_for_player(
    player_uuid: Uuid,
    lobby: &LobbyInfo,
    q_spawn_points: &Query<&Position, With<PlayerSpawnPoint>>,
) -> Position {
    let spawn_points: Vec<Position> = q_spawn_points.iter().copied().collect();
    let default_pos = Position {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        visual_priority: 0.0,
    };

    let Some(player_idx) = lobby
        .players
        .iter()
        .position(|player| player.player_uuid == player_uuid)
    else {
        warn!(
            "spawn_position_for_player: player {} missing from lobby ordering; using default position",
            player_uuid
        );
        return default_pos;
    };

    spawn_points
        .get(player_idx % spawn_points.len().max(1))
        .copied()
        .unwrap_or(default_pos)
}

fn spawn_requested_player(
    player: &LobbyPlayerInfo,
    lobby: &LobbyInfo,
    q_existing_sprites: &Query<&PlayerSprite>,
    q_pending_spawn: &Query<&PlayerSpawnRequest>,
    q_spawn_points: &Query<&Position, With<PlayerSpawnPoint>>,
    commands: &mut Commands,
) -> bool {
    if q_existing_sprites
        .iter()
        .any(|sprite| sprite.id == player.player_uuid)
    {
        info!(
            "spawn_requested_player: player {} already has an avatar; skipping duplicate join request",
            player.player_uuid
        );
        return false;
    }

    if q_pending_spawn
        .iter()
        .any(|request| request.player_uuid == player.player_uuid)
    {
        info!(
            "spawn_requested_player: player {} already has a pending spawn request",
            player.player_uuid
        );
        return false;
    }

    let spawn_pos = spawn_position_for_player(player.player_uuid, lobby, q_spawn_points);
    let net_id = NetworkId::from(player.player_uuid);
    let entity = commands
        .spawn((
            spawn_pos,
            PlayerSpawnRequest {
                player_uuid: player.player_uuid,
                network_id: net_id,
            },
            Replicated,
        ))
        .id();

    let owner_id = player.current_socket.unwrap_or(OwnerId::Server);
    if owner_id == OwnerId::Server {
        commands
            .entity(entity)
            .insert((Owner(OwnerId::Server), LocallyOwned));
    } else {
        commands.entity(entity).insert(Owner(owner_id));
    }

    info!(
        "spawn_requested_player: spawned avatar skeleton {:?} for player {} (owner={:?})",
        entity, player.player_uuid, owner_id
    );
    true
}

fn handle_request_join_mission(
    mut reader: MessageReader<FromClient<RequestJoinMission>>,
    q_lobby: Query<&LobbyInfo>,
    q_existing_sprites: Query<&PlayerSprite>,
    q_pending_spawn: Query<&PlayerSpawnRequest>,
    q_spawn_points: Query<&Position, With<PlayerSpawnPoint>>,
    uuid_map: Res<ClientUuidMap>,
    local_player: Option<Res<LocalPlayer>>,
    lobby_presence: Option<Res<LobbyPresenceRole>>,
    mut commands: Commands,
) {
    // TODO(multiplayer-first): remove once offline play uses a local in-memory transport.
    // With a loopback transport, the normal RequestJoinMission message flow would handle
    // this case and this entire branch could be deleted.
    // Offline single-player: no LobbyInfo entity exists by design (LobbyPresenceRole is absent).
    // Spawn the local player directly from the LocalPlayer resource and return early.
    if lobby_presence.is_none() {
        let has_messages = reader.read().count() > 0;
        if !has_messages {
            return;
        }
        let Some(local_player) = local_player else {
            warn!("handle_request_join_mission: offline mode but LocalPlayer resource missing");
            return;
        };
        let player_uuid = local_player.uuid;
        if q_existing_sprites.iter().any(|s| s.id == player_uuid)
            || q_pending_spawn.iter().any(|r| r.player_uuid == player_uuid)
        {
            return;
        }
        let spawn_pos = q_spawn_points.iter().next().copied().unwrap_or(Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            visual_priority: 0.0,
        });
        let net_id = NetworkId::from(player_uuid);
        commands.spawn((
            spawn_pos,
            PlayerSpawnRequest {
                player_uuid,
                network_id: net_id,
            },
            Replicated,
            Owner(OwnerId::Server),
            LocallyOwned,
        ));
        info!(
            "handle_request_join_mission: offline single-player spawn for {}",
            player_uuid
        );
        return;
    }

    let Ok(lobby) = q_lobby.single() else {
        warn!("handle_request_join_mission: missing LobbyInfo; cannot process join requests");
        return;
    };

    for msg in reader.read() {
        let requester_owner = to_owner_id(msg.client_id);
        let Some(requester_uuid) = uuid_map.0.get(&requester_owner).copied() else {
            warn!(
                "handle_request_join_mission: missing UUID mapping for {:?}; cannot honor join request",
                msg.client_id
            );
            continue;
        };

        let Some(player) = lobby
            .players
            .iter()
            .find(|player| player.player_uuid == requester_uuid)
        else {
            warn!(
                "handle_request_join_mission: requester {} is not present in LobbyInfo",
                requester_uuid
            );
            continue;
        };

        if player.current_socket != Some(requester_owner) && requester_owner != OwnerId::Server {
            warn!(
                "handle_request_join_mission: requester {} sent from mismatched owner {:?} (lobby has {:?})",
                requester_uuid, requester_owner, player.current_socket
            );
            continue;
        }

        if !player.connected && requester_owner != OwnerId::Server {
            warn!(
                "handle_request_join_mission: requester {} is not marked connected; queuing denied",
                requester_uuid
            );
            continue;
        }

        spawn_requested_player(
            player,
            lobby,
            &q_existing_sprites,
            &q_pending_spawn,
            &q_spawn_points,
            &mut commands,
        );
    }
}

/// Server: on entering TearingDown, remove the spawning-active marker and despawn
/// no gameplay entities. Domain plugins own their own teardown.
fn cleanup_player_spawning_flag(mut commands: Commands) {
    commands.remove_resource::<RepliconPlayerSpawningActive>();
}

fn handle_simulation_authority_requests(
    mut reader: MessageReader<FromClient<RequestSimulationAuthority>>,
    q_players: Query<&Owner, With<PlayerSprite>>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let requester_owner = to_owner_id(msg.client_id);
        let Ok(owner) = q_players.get(msg.message.entity) else {
            warn!(
                "handle_simulation_authority_requests: requested entity {:?} is not a player sprite",
                msg.message.entity
            );
            continue;
        };

        if owner.0 != requester_owner {
            warn!(
                "handle_simulation_authority_requests: requester {:?} attempted to claim entity {:?} owned by {:?}",
                requester_owner, msg.message.entity, owner.0
            );
            continue;
        }

        commands
            .entity(msg.message.entity)
            .insert(SimulationAuthorized);
    }
}

fn handle_simulation_authority_release(
    mut reader: MessageReader<FromClient<RelieveSimulationAuthority>>,
    q_players: Query<&Owner, With<PlayerSprite>>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let requester_owner = to_owner_id(msg.client_id);
        let Ok(owner) = q_players.get(msg.message.entity) else {
            warn!(
                "handle_simulation_authority_release: requested entity {:?} is not a player sprite",
                msg.message.entity
            );
            continue;
        };

        if owner.0 != requester_owner {
            warn!(
                "handle_simulation_authority_release: requester {:?} attempted to release entity {:?} owned by {:?}",
                requester_owner, msg.message.entity, owner.0
            );
            continue;
        }

        commands
            .entity(msg.message.entity)
            .remove::<SimulationAuthorized>();
    }
}

fn client_avatar_reconciliation_loop(
    q_owned_players: Query<
        (
            Entity,
            &PlayerSprite,
            &Owner,
            Has<SimulationAuthorized>,
            Has<LocallyOwned>,
        ),
        With<PlayerSprite>,
    >,
    local_player: Res<LocalPlayer>,
    app_state: Res<State<UIContextState>>,
    time: Res<Time>,
    mut commands: Commands,
    mut request_sim_authority: MessageWriter<RequestSimulationAuthority>,
    mut relieve_sim_authority: MessageWriter<RelieveSimulationAuthority>,
    mut last_sent_at: Local<HashMap<Entity, f32>>,
) {
    last_sent_at.retain(|entity, _| q_owned_players.get(*entity).is_ok());

    let local_uuid = local_player.uuid;
    let ready_for_local_simulation = *app_state.get() == UIContextState::InGame;
    let elapsed_secs = time.elapsed_secs();

    for (entity, player_sprite, owner, has_sim_authorized, has_locally_owned) in
        q_owned_players.iter()
    {
        if player_sprite.id != local_uuid {
            continue;
        }

        if !matches!(owner.0, OwnerId::Client(_)) {
            continue;
        }

        if ready_for_local_simulation {
            if has_sim_authorized && !has_locally_owned {
                commands.entity(entity).insert(LocallyOwned);
            }
            if !has_sim_authorized && throttle_ready(entity, elapsed_secs, &mut last_sent_at) {
                request_sim_authority.write(RequestSimulationAuthority { entity });
            }
        } else {
            if has_locally_owned {
                commands.entity(entity).remove::<LocallyOwned>();
            }
            if has_sim_authorized && throttle_ready(entity, elapsed_secs, &mut last_sent_at) {
                relieve_sim_authority.write(RelieveSimulationAuthority { entity });
            }
        }
    }
}
