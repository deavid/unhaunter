mod replication;

use bevy::prelude::*;
use bevy_replicon::prelude::{
    Channel, ClientId, ClientMessageAppExt, FromClient, Replicated, SendMode, ServerMessageAppExt,
    ToClients,
};
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use std::collections::{HashMap, HashSet};
use unboard_core::components::spawning::PlayerSpawnPoint;
use unmission_core::types::SimulationState;
use unplayer_core::components::{PlayerSpawnRequest, PlayerSprite};
use unreplicon_core::components::{
    LobbyInfo, LobbyPlayerInfo, NetworkEntityReady, OwnershipSentMarker,
    RepliconPlayerSpawningActive,
};
use unreplicon_core::messages::{
    FloorGearDespawnBroadcast, FloorGearSpawnBroadcast, OwnershipGranted, OwnershipRevoked,
    RequestJoinMission,
};
use unreplicon_core::network_id::NetworkId;
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unreplicon_core::resources::AuthorityRole;
use unreplicon_core::resources::{ClientUuidMap, LocalPlayerRole, is_pure_client};
use unspatial_core::position::Position;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
struct PendingOwnershipGrant {
    server_entity: Entity,
    frames_waited: u16,
}

#[derive(Resource, Default)]
struct PendingOwnershipGrantQueue(Vec<PendingOwnershipGrant>);

#[derive(Resource, Default)]
struct PendingMissionJoinRequests(HashSet<Uuid>);

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
    app.add_client_message::<RequestJoinMission>(Channel::Ordered);
    // Register server → client messages
    app.add_server_message::<FloorGearSpawnBroadcast>(Channel::Ordered);
    app.add_server_message::<FloorGearDespawnBroadcast>(Channel::Ordered);
    // NOTE: OwnershipGranted is registered as a plain (non-mapped) server message.
    // Using add_mapped_server_message would cause bevy_replicon to drop the message
    // silently if the referenced entity is not yet in ServerEntityMap at deserialization
    // time (which happens when the entity is initially hidden from the client).
    // handle_ownership_granted performs the entity map lookup manually.
    app.add_server_message::<OwnershipGranted>(Channel::Ordered);
    app.add_server_message::<OwnershipRevoked>(Channel::Ordered);

    app.init_resource::<PendingOwnershipGrantQueue>();
    app.init_resource::<PendingMissionJoinRequests>();

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
            process_pending_mission_join_requests,
            grant_ownership_when_ready,
            warn_on_stuck_pending_handover,
        )
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(SimulationState::Spawning).or(in_state(SimulationState::Ready)))
            .run_if(resource_exists::<RepliconPlayerSpawningActive>),
    );

    app.add_systems(
        Update,
        (
            handle_ownership_granted,
            handle_ownership_revoked,
            process_pending_ownership_grants,
        )
            .run_if(is_pure_client),
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

fn to_owner_id(client_id: ClientId) -> OwnerId {
    match client_id {
        ClientId::Server => OwnerId::Server,
        ClientId::Client(entity) => OwnerId::Client(entity),
    }
}

/// Helper: convert OwnerId to Replicon ClientId.
fn from_owner_id(owner_id: OwnerId) -> ClientId {
    match owner_id {
        OwnerId::Server => ClientId::Server,
        OwnerId::Client(e) => ClientId::Client(e),
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
    mut pending_join_requests: ResMut<PendingMissionJoinRequests>,
    mut commands: Commands,
) {
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

        let spawned = spawn_requested_player(
            player,
            lobby,
            &q_existing_sprites,
            &q_pending_spawn,
            &q_spawn_points,
            &mut commands,
        );

        if spawned {
            pending_join_requests.0.remove(&requester_uuid);
        } else {
            pending_join_requests.0.insert(requester_uuid);
        }
    }
}

fn process_pending_mission_join_requests(
    q_lobby: Query<&LobbyInfo>,
    q_existing_sprites: Query<&PlayerSprite>,
    q_pending_spawn: Query<&PlayerSpawnRequest>,
    q_spawn_points: Query<&Position, With<PlayerSpawnPoint>>,
    mut pending_join_requests: ResMut<PendingMissionJoinRequests>,
    mut commands: Commands,
) {
    if pending_join_requests.0.is_empty() {
        return;
    }

    let Ok(lobby) = q_lobby.single() else {
        warn!("process_pending_mission_join_requests: missing LobbyInfo; cannot drain join queue");
        return;
    };

    let queued_players: Vec<Uuid> = pending_join_requests.0.iter().copied().collect();
    for player_uuid in queued_players {
        let Some(player) = lobby
            .players
            .iter()
            .find(|player| player.player_uuid == player_uuid)
        else {
            warn!(
                "process_pending_mission_join_requests: queued player {} is no longer in LobbyInfo; dropping request",
                player_uuid
            );
            pending_join_requests.0.remove(&player_uuid);
            continue;
        };

        if !player.connected && player.current_socket.is_some() {
            debug!(
                "process_pending_mission_join_requests: queued player {} is not currently connected; keeping request queued",
                player_uuid
            );
            continue;
        }

        if spawn_requested_player(
            player,
            lobby,
            &q_existing_sprites,
            &q_pending_spawn,
            &q_spawn_points,
            &mut commands,
        ) {
            pending_join_requests.0.remove(&player_uuid);
        }
    }
}

/// Server: on entering TearingDown, remove the spawning-active marker and despawn
/// no gameplay entities. Domain plugins own their own teardown.
fn cleanup_player_spawning_flag(
    mut commands: Commands,
    mut pending_join_requests: ResMut<PendingMissionJoinRequests>,
) {
    pending_join_requests.0.clear();
    commands.remove_resource::<RepliconPlayerSpawningActive>();
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
    local_player: Option<Res<LocalPlayerRole>>,
    mut commands: Commands,
) {
    let is_dedicated_server = local_player.is_none();
    for (entity, owner, has_ready_marker) in q_ready.iter() {
        if !is_dedicated_server && !player_ready_for_handover(has_ready_marker) {
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
    local_player: Option<Res<LocalPlayerRole>>,
    mut wait_frames_by_entity: Local<HashMap<Entity, u16>>,
) {
    if local_player.is_none() {
        return; // Dedicated servers don't use NetworkEntityReady
    }
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

/// Client: Handle ownership revocation — remove `LocallyOwned` from the entity.
fn handle_ownership_revoked(
    mut reader: MessageReader<OwnershipRevoked>,
    entity_map: Res<ServerEntityMap>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let server_entity = msg.entity;
        if let Some(client_entity) = entity_map.to_client().get(&server_entity).copied() {
            info!(
                "handle_ownership_revoked: removing LocallyOwned from client {:?} (server {:?})",
                client_entity, server_entity
            );
            commands.entity(client_entity).remove::<LocallyOwned>();
        } else {
            warn!(
                "handle_ownership_revoked: no client mapping for server entity {:?}",
                server_entity
            );
        }
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
