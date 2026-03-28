mod replication;

use bevy::prelude::*;
use bevy_replicon::prelude::{
    Channel, ClientId, ClientMessageAppExt, FromClient, Replicated, SendMode, ServerMessageAppExt,
    ToClients,
};
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use std::collections::HashMap;
use unbehavior_core::behavior::Behavior;
use unbehavior_core::behavior::Interactive;
use unbehavior_core::components::FloorItemCollidable;
use unboard_core::components::spawning::PlayerSpawnPoint;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::messages::{TruckLoadoutAction, TruckLoadoutMessage};
use ungear_core::resources::spawner::{GearMarker, GearSpawnerRegistry};
use ungear_core::types::gear::equipment::{EquipmentPosition, Hand};
use ungear_core::types::gear::kind::GearKind;
use uninteraction_core::events::InteractionRequestMessage;
use uninteraction_core::interaction::ExecuteInteractionEvent;
use unmission_core::types::SimulationState;
use unorchestrator_core::UIContextState;
use unplayer_core::components::{PlayerSpawnRequest, PlayerSprite};
use unreplicon_core::components::{
    LobbyInfo, NetworkEntityReady, OwnershipSentMarker, RepliconPlayerSpawningActive,
};
use unreplicon_core::messages::{
    FloorGearDespawnBroadcast, FloorGearSpawnBroadcast, HostMovableMotionEvent,
    MovableMotionBroadcast, OwnershipGranted, RequestDrop, RequestGrab, SaltDroppedMessage,
};
use unreplicon_core::network_id::NetworkId;
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unreplicon_core::resources::AuthorityRole;
use unreplicon_core::resources::{LocalPlayerRole, is_pure_client};
use unspatial_core::boardposition::{BoardPosition, MapEntityFieldBPos};
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;

#[derive(Debug, Clone, Copy)]
struct PendingOwnershipGrant {
    server_entity: Entity,
    frames_waited: u16,
}

#[derive(Resource, Default)]
struct PendingOwnershipGrantQueue(Vec<PendingOwnershipGrant>);

// SRE SRE SRE SRE SRE
#[allow(clippy::manual_is_multiple_of)]
fn sre_telemetry(
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
        "SRE TELEMETRY TICK ({}): auth={} local={} spawning={} state={:?}",
        *frames,
        authority.is_some(),
        local_player.is_some(),
        spawning_active.is_some(),
        state.map(|s| *s.get())
    );
    for (e, req, spr, own, rdy, sent, locown) in q.iter() {
        if req.is_some() || spr.is_some() {
            info!(
                "SRE ENTITY {:?}: req={} spr={} owner={:?} ready={} sent={} locally_owned={}",
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
    app.add_client_message::<InteractionRequestMessage>(Channel::Ordered);
    app.add_client_message::<TruckLoadoutMessage>(Channel::Ordered);
    app.add_client_message::<SaltDroppedMessage>(Channel::Ordered);
    app.add_mapped_client_message::<RequestGrab>(Channel::Ordered);
    app.add_mapped_client_message::<RequestDrop>(Channel::Ordered);
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

    // SRE Telemetry
    app.add_systems(Update, sre_telemetry);

    // Host/offline: spawn and tag player entities when InGame starts.
    // Gated by AuthorityRole so it runs on Host and Dedicated Server.
    app.add_systems(
        OnEnter(SimulationState::Spawning),
        setup_mission_players.run_if(resource_exists::<AuthorityRole>),
    );

    // Server-side: message handlers + net state sync.
    app.add_systems(
        Update,
        (
            handle_interaction_request,
            broadcast_movable_motion,
            handle_request_grab,
            handle_request_drop,
            handle_salt_drop,
        )
            .run_if(resource_exists::<AuthorityRole>),
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

    // Server-side: truck loadout message handler (Truck phase only)
    app.add_systems(
        Update,
        handle_truck_loadout_message
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(SimulationState::Ready)),
    );

    // Client-side: receive ownership, either resolving it immediately or deferring
    app.add_systems(
        Update,
        (handle_ownership_granted, process_pending_ownership_grants).run_if(is_pure_client),
    );

    // Client-side: gear propagation (needs UIContextState::InGame to avoid spam/early execution)
    app.add_systems(
        Update,
        (propagate_gear_ownership, cleanup_gear_ownership)
            .run_if(in_state(UIContextState::InGame))
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
        ClientId::Client(e) => OwnerId::Client(e),
    }
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
            "SRE: setup_mission_players is processing lobby player uuid={} idx={}",
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

/// Server: handle `SaltDroppedMessage` from join clients.
///
/// Spawns a `Replicated` `SaltPile` entity at the reported position so that
/// bevy_replicon broadcasts it to all connected clients.
fn handle_salt_drop(
    mut reader: MessageReader<FromClient<SaltDroppedMessage>>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let [x, y, z, visual_priority] = msg.message.pos;
        let pos = unspatial_core::position::Position {
            x,
            y,
            z,
            visual_priority,
        };
        commands.spawn((
            ungearitems_core::components::salt::SaltPile,
            pos,
            Replicated,
        ));
    }
}

/// Server: handle `InteractionRequestMessage` from connected clients.
fn handle_interaction_request(
    mut reader: MessageReader<FromClient<InteractionRequestMessage>>,
    q_interactive: Query<(Entity, &MapEntityFieldBPos), With<Interactive>>,
    mut ev_interact: MessageWriter<ExecuteInteractionEvent>,
) {
    for msg in reader.read() {
        let target_bpos = BoardPosition {
            x: msg.message.position[0] as i64,
            y: msg.message.position[1] as i64,
            z: msg.message.position[2] as i64,
        };

        let found = q_interactive
            .iter()
            .find(|(_, bpos)| bpos.0 == target_bpos)
            .map(|(e, _)| e);

        if let Some(entity) = found {
            info!(
                "SERVER: Validated door interaction at {:?} from client {:?}",
                target_bpos, msg.client_id
            );
            info!(
                "SERVER: Firing ExecuteInteractionEvent for entity {:?} (ietype={:?}, force_tuid={:?})",
                entity, msg.message.ietype, msg.message.force_tuid
            );
            ev_interact.write(ExecuteInteractionEvent {
                entity,
                ietype: msg.message.ietype.clone(),
                force_tuid: msg.message.force_tuid,
            });
            // Server processes the interaction, mutating Behavior.
            // bevy_replicon replicates the changed Behavior to all clients.
        } else {
            warn!(
                "SERVER: Failed to find interactive entity at {:?}",
                target_bpos
            );
        }
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

fn handle_request_grab(
    mut reader: MessageReader<FromClient<RequestGrab>>,
    mut commands: Commands,
    q_items: Query<
        (Entity, Option<&Owner>, Has<GearKind>, Has<Behavior>),
        With<FloorItemCollidable>,
    >,
) {
    for msg in reader.read() {
        let client_id = msg.client_id;
        let item_entity = msg.message.entity;

        if let Ok((entity, owner, is_gear, is_furniture)) = q_items.get(item_entity)
            && owner.is_none()
        {
            let owner_id = to_owner_id(client_id);
            commands.entity(entity).insert(Owner(owner_id));
            commands.entity(entity).remove::<FloorItemCollidable>();
            commands
                .entity(entity)
                .remove::<ungear_core::components::deployedgear::DeployedGear>();

            if is_gear {
                // Gear visual cleanup is handled reactively on clients.
            }

            if is_furniture {
                // Furniture keeps its visuals while carried.
            }

            if client_id == bevy_replicon::prelude::ClientId::Server {
                commands
                    .entity(entity)
                    .insert(unreplicon_core::ownership::LocallyOwned);
            } else {
                commands.write_message(ToClients {
                    mode: SendMode::Direct(client_id),
                    message: OwnershipGranted { entity },
                });
            }
        }
    }
}

fn handle_request_drop(
    mut reader: MessageReader<FromClient<RequestDrop>>,
    mut commands: Commands,
    q_items: Query<(Entity, &Owner, Has<GearKind>, Has<Behavior>)>,
) {
    for msg in reader.read() {
        let client_id = msg.client_id;
        let item_entity = msg.message.entity;

        if let Ok((entity, owner, is_gear, is_furniture)) = q_items.get(item_entity)
            && from_owner_id(owner.0) == client_id
        {
            commands.entity(entity).remove::<Owner>();
            commands.entity(entity).insert(FloorItemCollidable);
            commands.entity(entity).insert(Position {
                x: msg.message.position[0],
                y: msg.message.position[1],
                z: msg.message.position[2],
                ..Default::default()
            });

            if is_gear {
                commands.entity(entity).insert(DeployedGear {
                    direction: Direction {
                        dx: msg.message.direction[0],
                        dy: msg.message.direction[1],
                        dz: msg.message.direction[2],
                    },
                });
                commands.entity(entity).insert(EquipmentPosition::Deployed);
            }

            if is_furniture {
                // Furniture visuals are preserved while carried; no extra components needed.
            }
        }
    }
}

fn handle_truck_loadout_message(
    mut reader: MessageReader<FromClient<TruckLoadoutMessage>>,
    mut q_players: Query<(&Owner, &mut PlayerGear)>,
    gear_registry: Res<GearSpawnerRegistry>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let sender_id = msg.client_id;
        info!(
            "TRUCK_NET: Received action {:?} from client {:?}",
            msg.message.action, sender_id
        );

        let Some((owner, mut p_gear)) = q_players.iter_mut().find_map(|(owner, gear)| {
            if crate::systems::players::from_owner_id(owner.0) == sender_id {
                Some((owner, gear))
            } else {
                None
            }
        }) else {
            warn!(
                "TRUCK_NET_FAIL: Could not find PlayerGear component for client {:?}",
                sender_id
            );
            continue;
        };

        match msg.message.action {
            TruckLoadoutAction::AddGear(kind) => {
                let has_space = p_gear.left_hand.is_none()
                    || p_gear.right_hand.is_none()
                    || p_gear.inventory.len() < 2;
                if !has_space {
                    warn!(
                        "TRUCK_NET_FAIL: Client {:?} AddGear but inventory is full on server!",
                        sender_id
                    );
                    continue;
                }
                let entity = gear_registry.spawn(&mut commands, kind);
                let rng_val = uncommon_app_core::random_seed::heavy_rng_seed();
                let net_id = unreplicon_core::network_id::NetworkId(rng_val.max(1000));

                commands.entity(entity).insert((
                    bevy_replicon::prelude::Replicated,
                    Owner(owner.0),
                    net_id,
                ));

                let client_id = crate::systems::players::from_owner_id(owner.0);
                if client_id != bevy_replicon::prelude::ClientId::Server {
                    commands.write_message(bevy_replicon::prelude::ToClients {
                        mode: bevy_replicon::prelude::SendMode::Direct(client_id),
                        message: unreplicon_core::messages::OwnershipGranted { entity },
                    });
                }

                if p_gear.left_hand.is_none() {
                    p_gear.left_hand = Some(entity);
                } else if p_gear.right_hand.is_none() {
                    p_gear.right_hand = Some(entity);
                } else {
                    p_gear.inventory.push(entity);
                }
                info!(
                    "TRUCK_NET: Spawned {:?} and added to PlayerGear for {:?}",
                    entity, sender_id
                );
            }
            TruckLoadoutAction::ClearHand(hand) => {
                let entity = match hand {
                    Hand::Left => p_gear.left_hand.take(),
                    Hand::Right => p_gear.right_hand.take(),
                };
                if let Some(e) = entity {
                    info!("TRUCK_NET: Despawning entity {:?} from hand {:?}", e, hand);
                    commands.entity(e).despawn();
                } else {
                    warn!(
                        "TRUCK_NET_FAIL: Client {:?} requested ClearHand({:?}) but server's PlayerGear slot was ALREADY EMPTY!",
                        sender_id, hand
                    );
                    warn!(
                        "TRUCK_NET_STATE: Server thinks gear is -> left: {:?}, right: {:?}, inv: {:?}",
                        p_gear.left_hand, p_gear.right_hand, p_gear.inventory
                    );
                }
            }
            TruckLoadoutAction::ClearInventorySlot(idx) => {
                if idx < p_gear.inventory.len() {
                    let e = p_gear.inventory.remove(idx);
                    info!(
                        "TRUCK_NET: Despawning entity {:?} from inventory slot {}",
                        e, idx
                    );
                    commands.entity(e).despawn();
                } else {
                    warn!(
                        "TRUCK_NET_FAIL: Client {:?} requested ClearInventorySlot({}), but server's inventory length is {}",
                        sender_id,
                        idx,
                        p_gear.inventory.len()
                    );
                }
            }
        }
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
            "SRE: grant_ownership_when_ready checked entity {:?} with owner={:?} (client_id={:?})",
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
                "SRE: grant_ownership_when_ready skipping host-owned entity {:?}",
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

fn apply_local_ownership_and_propagate_gear(
    client_entity: Entity,
    commands: &mut Commands,
    q_player_gear: &Query<&PlayerGear>,
) {
    commands.entity(client_entity).insert(LocallyOwned);

    // Also grant LocallyOwned to all gear entities this player holds.
    if let Ok(gear) = q_player_gear.get(client_entity) {
        if let Some(entity) = gear.left_hand {
            commands.entity(entity).insert(LocallyOwned);
            debug!(
                "handle_ownership_granted: propagating LocallyOwned to left_hand {:?}",
                entity
            );
        }
        if let Some(entity) = gear.right_hand {
            commands.entity(entity).insert(LocallyOwned);
            debug!(
                "handle_ownership_granted: propagating LocallyOwned to right_hand {:?}",
                entity
            );
        }
        for &entity in &gear.inventory {
            commands.entity(entity).insert(LocallyOwned);
            debug!(
                "handle_ownership_granted: propagating LocallyOwned to inventory item {:?}",
                entity
            );
        }
    }
}

/// Client: Handle ownership granted.
fn handle_ownership_granted(
    mut reader: MessageReader<OwnershipGranted>,
    entity_map: Res<ServerEntityMap>,
    mut commands: Commands,
    q_player_gear: Query<&PlayerGear>,
    mut pending_grants: ResMut<PendingOwnershipGrantQueue>,
) {
    for msg in reader.read() {
        let server_entity = msg.entity;
        info!(
            "SRE: handle_ownership_granted received OwnershipGranted for server entity {:?}",
            server_entity
        );

        if let Some(client_entity) = entity_map.to_client().get(&server_entity).copied() {
            info!(
                "handle_ownership_granted: mapped server {:?} to client {:?}",
                server_entity, client_entity
            );
            apply_local_ownership_and_propagate_gear(client_entity, &mut commands, &q_player_gear);
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
    q_player_gear: Query<&PlayerGear>,
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
            apply_local_ownership_and_propagate_gear(client_entity, &mut commands, &q_player_gear);
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

/// Client-side: propagate the `LocallyOwned` marker to all gear entities in local player slots.
/// Runs constantly but filters on gear `Without<LocallyOwned>` to avoid redundant commands,
/// catching gear entities that arrive/hydrate after the player entity is already setup.
fn propagate_gear_ownership(
    mut commands: Commands,
    player_q: Query<(&PlayerGear, &LocallyOwned)>,
    gear_q: Query<Entity, (With<GearMarker>, Without<LocallyOwned>)>,
) {
    for (player_gear, _locally_owned) in player_q.iter() {
        let gear_entities = player_gear
            .left_hand
            .into_iter()
            .chain(player_gear.right_hand)
            .chain(player_gear.inventory.iter().copied());

        for gear_entity in gear_entities {
            if gear_q.get(gear_entity).is_ok() {
                commands.entity(gear_entity).insert(LocallyOwned);
                info!(
                    "propagate_gear_ownership: Added LocallyOwned to gear entity {:?}",
                    gear_entity
                );
            }
        }
    }
}

/// Client-side: when a player entity's `LocallyOwned` is removed (or the component is changed
/// and it no longer exists), remove it from the gear entities as well.
fn cleanup_gear_ownership(
    mut commands: Commands,
    mut removed: RemovedComponents<LocallyOwned>,
    player_q: Query<&PlayerGear>,
    gear_q: Query<Entity, (With<GearMarker>, With<LocallyOwned>)>,
) {
    for player_entity in removed.read() {
        if let Ok(player_gear) = player_q.get(player_entity) {
            let gear_entities = player_gear
                .left_hand
                .into_iter()
                .chain(player_gear.right_hand)
                .chain(player_gear.inventory.iter().copied());

            for gear_entity in gear_entities {
                if gear_q.get(gear_entity).is_ok() {
                    commands.entity(gear_entity).remove::<LocallyOwned>();
                    debug!(
                        "cleanup_gear_ownership: Removed LocallyOwned from gear entity {:?}",
                        gear_entity
                    );
                }
            }
        }
    }
}
