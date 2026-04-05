mod replication;

use bevy::prelude::*;
use bevy_replicon::prelude::{
    Channel, ClientId, Replicated, SendMode, ServerMessageAppExt, ToClients,
};
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use std::collections::HashMap;
use unboard_core::components::spawning::PlayerSpawnPoint;
use unmission_core::types::SimulationState;
use unplayer_core::components::{PlayerSpawnRequest, PlayerSprite};
use unreplicon_core::components::{
    LobbyInfo, NetworkEntityReady, OwnershipSentMarker, RepliconPlayerSpawningActive,
};
use unreplicon_core::messages::{
    FloorGearDespawnBroadcast, FloorGearSpawnBroadcast, HostMovableMotionEvent,
    MovableMotionBroadcast, OwnershipGranted,
};
use unreplicon_core::network_id::NetworkId;
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unreplicon_core::resources::AuthorityRole;
use unreplicon_core::resources::{LocalPlayerRole, is_pure_client};
use unspatial_core::position::Position;

#[derive(Debug, Clone, Copy)]
struct PendingOwnershipGrant {
    server_entity: Entity,
    frames_waited: u16,
}

#[derive(Resource, Default)]
struct PendingOwnershipGrantQueue(Vec<PendingOwnershipGrant>);

// Periodic diagnostics for player spawn and ownership handoff state.
#[allow(clippy::manual_is_multiple_of)]
fn player_spawn_telemetry(
    q: Query<(
        Entity,
        Option<&PlayerSpawnRequest>,
        Option<&PlayerSprite>,
        Option<&Owner>,
        Option<&NetworkEntityReady>,
        Option<&OwnershipSentMarker>,
        Option<&LocallyOwned>,
    )>,
    authority: Option<Res<AuthorityRole>>,
    local_player: Option<Res<LocalPlayerRole>>,
    spawning_active: Option<Res<RepliconPlayerSpawningActive>>,
    state: Option<Res<State<SimulationState>>>,
    mut frames: Local<u32>,
) {
    *frames += 1;
    if *frames % 60 != 0 {
        return;
    }

    info!(
        "player_spawn_telemetry: tick={} auth={} local_player={} spawning_active={} state={:?}",
        *frames,
        authority.is_some(),
        local_player.is_some(),
        spawning_active.is_some(),
        state.map(|s| *s.get())
    );
    for (e, req, spr, own, rdy, sent, locown) in q.iter() {
        if req.is_some() || spr.is_some() {
            info!(
                "player_spawn_telemetry: entity {:?} spawn_request={} player_sprite={} owner={:?} network_ready={} ownership_sent={} locally_owned={}",
                e,
                req.is_some(),
                spr.is_some(),
                own.map(|o| o.0),
                rdy.is_some(),
                sent.is_some(),
                locown.is_some()
            );
        }
    }
}

pub(super) fn app_setup(app: &mut App) {
    // Register client → server messages
    // Register server → client messages
    app.add_mapped_server_message::<MovableMotionBroadcast>(Channel::Ordered);
    app.add_server_message::<FloorGearSpawnBroadcast>(Channel::Ordered);
    app.add_server_message::<FloorGearDespawnBroadcast>(Channel::Ordered);
    // NOTE: OwnershipGranted is registered as a plain (non-mapped) server message.
    // Using add_mapped_server_message would cause bevy_replicon to drop the message
    // silently if the referenced entity is not yet in ServerEntityMap at deserialization
    // time (which happens when the entity is initially hidden from the client).
    // handle_ownership_granted performs the entity map lookup manually.
    app.add_server_message::<OwnershipGranted>(Channel::Ordered);

    // Register local messages
    app.add_message::<HostMovableMotionEvent>();
    app.init_resource::<PendingOwnershipGrantQueue>();

    replication::app_setup(app);

    // Periodic diagnostics for player spawning and ownership handoff.
    app.add_systems(Update, player_spawn_telemetry);

    // Host/offline: spawn and tag player entities when InGame starts.
    // Gated by AuthorityRole so it runs on Host and Dedicated Server.
    app.add_systems(
        OnEnter(SimulationState::Spawning),
        setup_mission_players.run_if(resource_exists::<AuthorityRole>),
    );

    // Server-side: message handlers + net state sync.
    app.add_systems(
        Update,
        broadcast_movable_motion.run_if(resource_exists::<AuthorityRole>),
    );

    // Server-side: spawn player entities for clients that joined after mission start.
    app.add_systems(
        Update,
        (
            spawn_late_joining_players,
            grant_ownership_when_ready,
            warn_on_stuck_pending_handover,
        )
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(SimulationState::Ready))
            .run_if(resource_exists::<RepliconPlayerSpawningActive>),
    );

    app.add_systems(
        Update,
        (handle_ownership_granted, process_pending_ownership_grants).run_if(is_pure_client),
    );

    // Cleanup the spawning-active marker when leaving InGame.
    app.add_systems(
        OnEnter(SimulationState::TearingDown),
        cleanup_player_spawning_flag,
    );
}

fn player_ready_for_handover(has_ready_marker: bool) -> bool {
    has_ready_marker
}

/// Helper: convert OwnerId to Replicon ClientId.
fn from_owner_id(owner_id: OwnerId) -> ClientId {
    match owner_id {
        OwnerId::Server => ClientId::Server,
        OwnerId::Client(e) => ClientId::Client(e),
    }
}

/// Server: called once on `OnEnter(SimulationState::Spawning)`.
/// Spawns a minimal network skeleton for each lobby player.
/// Domain components (vitals, locomotion, gear, etc.) are inserted by each
/// domain's own hydration system reacting to `Added<PlayerSprite>`.
fn setup_mission_players(
    q_lobby: Query<&LobbyInfo>,
    q_spawn_points: Query<&Position, With<PlayerSpawnPoint>>,
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

    let Ok(lobby) = q_lobby.single() else {
        warn!("setup_mission_players: no LobbyInfo entity found — skipping player spawn");
        return;
    };

    for (idx, player) in lobby.players.iter().enumerate() {
        // Pick a deterministic spawn point based on lobby order.
        let spawn_pos = spawn_points
            .get(idx % spawn_points.len().max(1))
            .copied()
            .unwrap_or(default_pos);

        let net_id = NetworkId::from(player.player_uuid);
        info!(
            "setup_mission_players: processing lobby player uuid={} idx={}",
            player.player_uuid, idx
        );

        // Spawn a thin request entity. unplayer-plugin materializes PlayerSprite
        // and the canonical player baseline components.
        let entity_commands = commands.spawn((
            spawn_pos,
            PlayerSpawnRequest {
                player_uuid: player.player_uuid,
                network_id: net_id,
            },
            Replicated,
        ));
        let entity = entity_commands.id();

        let is_host = player.current_socket.is_none();

        if is_host {
            // Host player: authority owns it locally.
            commands
                .entity(entity)
                .insert((Owner(OwnerId::Server), LocallyOwned));

            info!(
                "setup_mission_players: spawned host skeleton {:?} for player {}",
                entity, player.player_uuid
            );
        } else {
            let socket_owner_id = player.current_socket.unwrap();
            commands.entity(entity).insert(Owner(socket_owner_id));

            // NOTE: We no longer send OwnershipGranted immediately.
            // The grant_ownership_when_ready system will send it once the entity
            // is fully hydrated (e.g. has PlayerGear).
            info!(
                "setup_mission_players: spawned remote skeleton {:?} for player {} (owner={:?})",
                entity, player.player_uuid, socket_owner_id
            );
        }
    }
}

/// Server: on entering TearingDown, remove the spawning-active marker and despawn
/// no gameplay entities. Domain plugins own their own teardown.
fn cleanup_player_spawning_flag(mut commands: Commands) {
    commands.remove_resource::<RepliconPlayerSpawningActive>();
}

/// Server: runs every frame during InGame to spawn player entities for clients that
/// connected (and were added to lobby.players) after setup_mission_players already ran.
/// Domain plugins react to Added<PlayerSprite> to insert their own components.
fn spawn_late_joining_players(
    q_lobby: Query<&LobbyInfo>,
    q_existing_sprites: Query<&PlayerSprite>,
    q_pending_spawn: Query<&PlayerSpawnRequest>,
    q_spawn_points: Query<&Position, With<unboard_core::components::spawning::PlayerSpawnPoint>>,
    mut commands: Commands,
) {
    let Ok(lobby) = q_lobby.single() else {
        return;
    };

    let spawn_points: Vec<Position> = q_spawn_points.iter().copied().collect();
    let default_pos = Position {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        visual_priority: 0.0,
    };

    for (idx, player) in lobby.players.iter().enumerate() {
        // Only handle remote clients that are currently connected.
        if player.current_socket.is_none() || !player.connected {
            continue;
        }

        // Skip if a PlayerSprite already exists for this player UUID.
        if q_existing_sprites
            .iter()
            .any(|s| s.id == player.player_uuid)
        {
            continue;
        }

        // Skip if a request entity is already pending materialization.
        if q_pending_spawn
            .iter()
            .any(|r| r.player_uuid == player.player_uuid)
        {
            continue;
        }

        let spawn_pos = spawn_points
            .get(idx % spawn_points.len().max(1))
            .copied()
            .unwrap_or(default_pos);

        let net_id = NetworkId::from(player.player_uuid);
        let socket_owner_id = player.current_socket.unwrap();

        let entity_commands = commands.spawn((
            spawn_pos,
            PlayerSpawnRequest {
                player_uuid: player.player_uuid,
                network_id: net_id,
            },
            Owner(socket_owner_id),
            Replicated,
        ));
        let entity = entity_commands.id();

        // NOTE: We no longer send OwnershipGranted immediately.
        // The grant_ownership_when_ready system will send it once the entity
        // is fully hydrated (e.g. has PlayerGear).
        info!(
            "spawn_late_joining_players: spawned skeleton {:?} for late-joining player {} (owner={:?})",
            entity, player.player_uuid, socket_owner_id
        );
    }
}

/// Server: broadcast a ghost-induced movable-object motion.
fn broadcast_movable_motion(
    mut reader: MessageReader<HostMovableMotionEvent>,
    mut ev_broadcast: MessageWriter<ToClients<MovableMotionBroadcast>>,
) {
    for msg in reader.read() {
        ev_broadcast.write(ToClients {
            mode: SendMode::Broadcast,
            message: MovableMotionBroadcast {
                entity: msg.entity,
                start: msg.start,
                end: msg.end,
                duration: msg.duration,
                ease: msg.ease,
            },
        });
    }
}

/// Server: Watches for entities that have been stamped as fully hydrated by the domains.
/// Once ready, it sends the OwnershipGranted packet so the client can safely take control
/// without activating its `noop_write` shields too early.
///
/// Matches both `PlayerSprite` (host/offline, fully materialized) and `PlayerSpawnRequest`
/// (dedicated server, where no player-plugin hydration runs).
fn grant_ownership_when_ready(
    q_ready: Query<
        (Entity, &Owner, Has<NetworkEntityReady>),
        (
            Or<(With<PlayerSprite>, With<PlayerSpawnRequest>)>,
            Without<OwnershipSentMarker>,
        ),
    >,
    mut commands: Commands,
) {
    for (entity, owner, has_ready_marker) in q_ready.iter() {
        if !player_ready_for_handover(has_ready_marker) {
            continue;
        }

        // Mark that we've sent it so we don't spam the network
        commands.entity(entity).insert(OwnershipSentMarker);

        let client_id = from_owner_id(owner.0);
        info!(
            "grant_ownership_when_ready: evaluated entity {:?} with owner={:?} (client_id={:?})",
            entity, owner.0, client_id
        );

        if client_id != bevy_replicon::prelude::ClientId::Server {
            commands.write_message(ToClients {
                mode: SendMode::Direct(client_id),
                message: OwnershipGranted { entity },
            });
            info!(
                "grant_ownership_when_ready: Entity {:?} is fully hydrated. Ownership handed to {:?}",
                entity, owner.0
            );
        } else {
            info!(
                "grant_ownership_when_ready: skipping host-owned entity {:?}",
                entity
            );
        }
    }
}

/// Server: diagnostic watchdog for player entities that remain pending handover.
/// Matches both `PlayerSprite` (host) and `PlayerSpawnRequest` (dedicated server).
fn warn_on_stuck_pending_handover(
    q_pending: Query<
        (Entity, &Owner),
        (
            Or<(With<PlayerSprite>, With<PlayerSpawnRequest>)>,
            Without<NetworkEntityReady>,
            Without<OwnershipSentMarker>,
        ),
    >,
    mut wait_frames_by_entity: Local<HashMap<Entity, u16>>,
) {
    wait_frames_by_entity.retain(|entity, _| q_pending.get(*entity).is_ok());

    for (entity, owner) in q_pending.iter() {
        let entry = wait_frames_by_entity.entry(entity).or_insert(0);
        *entry = entry.saturating_add(1);

        if *entry == 120 || *entry == 600 {
            warn!(
                "warn_on_stuck_pending_handover: player {:?} still pending handover after {} frames (owner={:?})",
                entity, *entry, owner.0
            );
        }
    }
}

fn apply_local_ownership(client_entity: Entity, commands: &mut Commands) {
    commands.entity(client_entity).insert(LocallyOwned);
}

/// Client: Handle ownership granted.
fn handle_ownership_granted(
    mut reader: MessageReader<OwnershipGranted>,
    entity_map: Res<ServerEntityMap>,
    mut commands: Commands,
    mut pending_grants: ResMut<PendingOwnershipGrantQueue>,
) {
    for msg in reader.read() {
        let server_entity = msg.entity;
        info!(
            "handle_ownership_granted: received OwnershipGranted for server entity {:?}",
            server_entity
        );

        if let Some(client_entity) = entity_map.to_client().get(&server_entity).copied() {
            info!(
                "handle_ownership_granted: mapped server {:?} to client {:?}",
                server_entity, client_entity
            );
            apply_local_ownership(client_entity, &mut commands);
            continue;
        }

        let already_queued = pending_grants
            .0
            .iter()
            .any(|pending| pending.server_entity == server_entity);
        if already_queued {
            debug!(
                "handle_ownership_granted: server entity {:?} already queued for deferred mapping",
                server_entity
            );
            continue;
        }

        warn!(
            "handle_ownership_granted: no client mapping yet for server entity {:?}; deferring ownership",
            server_entity
        );
        pending_grants.0.push(PendingOwnershipGrant {
            server_entity,
            frames_waited: 0,
        });
    }
}

/// Client: retries ownership grants that arrived before ServerEntityMap contained the entity.
fn process_pending_ownership_grants(
    entity_map: Res<ServerEntityMap>,
    mut pending_grants: ResMut<PendingOwnershipGrantQueue>,
    mut commands: Commands,
) {
    if pending_grants.0.is_empty() {
        return;
    }

    let mut still_pending = Vec::new();
    for mut pending in pending_grants.0.drain(..) {
        if let Some(client_entity) = entity_map.to_client().get(&pending.server_entity).copied() {
            info!(
                "process_pending_ownership_grants: resolved server {:?} -> client {:?} after {} frames",
                pending.server_entity, client_entity, pending.frames_waited
            );
            apply_local_ownership(client_entity, &mut commands);
            continue;
        }

        pending.frames_waited = pending.frames_waited.saturating_add(1);
        if pending.frames_waited == 120 || pending.frames_waited == 600 {
            warn!(
                "process_pending_ownership_grants: still waiting for mapping of server entity {:?} ({} frames)",
                pending.server_entity, pending.frames_waited
            );
        }
        still_pending.push(pending);
    }

    pending_grants.0 = still_pending;
}
