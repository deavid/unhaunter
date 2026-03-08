use bevy::prelude::*;
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientId, ClientMessageAppExt, FromClient, Replicated, SendMode,
    ServerMessageAppExt, ToClients,
};
use bevy_replicon::server::visibility::client_visibility::ClientVisibility;
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use unbehavior::components::{FloorItemCollidable, TmxEntityId};
use unboard_core::components::spawning::PlayerSpawnPoint;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unfoundation_core::types::gear::Hand;
use ungear_core::components::playergear::HeldObject;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::{GearHydrated, GearMarker, GearSpawnerRegistry};
use ungear_core::types::gear::kind::GearKind;
use uninteraction_core::interaction::ExecuteInteractionEvent;
use unplayer_core::components::{Hiding, MainPlayer, PlayerSpectating, PlayerSprite, Stamina};
use unrender_std::components::visuals::Viewer;
use unrender_std::resources::visibility_data::VisibilityData;
use unreplicon_core::components::{LobbyInfo, RepliconPlayerSpawningActive};
use unreplicon_core::messages::{
    ExportGearStateMessage, ExportStateMessage, FloorGearDespawnBroadcast, FloorGearSpawnBroadcast,
    HostFloorGearDroppedEvent, HostFloorGearPickedUpEvent, HostInteractionOccurred,
    HostMovableMotionEvent, InteractionRequestMessage, MovableMotionBroadcast, OwnershipGranted,
    OwnershipReleased, RemoteInteractionBroadcast, RequestPickupGear, TruckLoadoutAction,
    TruckLoadoutMessage,
};
use unreplicon_core::network_id::NetworkId;
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unreplicon_core::resources::LocalPlayer;
use unspatial_core::boardposition::{BoardPosition, MapEntityFieldBPos};
use unspatial_core::position::Position;
use untypes_core::roles::is_pure_client;
use untypes_core::roles::{AuthorityRole, LocalPlayerRole};
use untypes_core::states::{AppState, GameState};

pub(super) fn app_setup(app: &mut App) {
    // Register client → server messages
    app.add_client_message::<ExportStateMessage>(Channel::Unreliable);
    app.add_client_message::<InteractionRequestMessage>(Channel::Ordered);
    app.add_client_message::<TruckLoadoutMessage>(Channel::Ordered);
    app.add_mapped_client_message::<RequestPickupGear>(Channel::Ordered);
    app.add_mapped_client_message::<OwnershipReleased>(Channel::Ordered);
    app.add_mapped_client_message::<ExportGearStateMessage>(Channel::Unreliable);
    // Register server → client messages
    app.add_server_message::<RemoteInteractionBroadcast>(Channel::Ordered);
    app.add_server_message::<MovableMotionBroadcast>(Channel::Ordered);
    app.add_server_message::<FloorGearSpawnBroadcast>(Channel::Ordered);
    app.add_server_message::<FloorGearDespawnBroadcast>(Channel::Ordered);
    // NOTE: OwnershipGranted is registered as a plain (non-mapped) server message.
    // Using add_mapped_server_message would cause bevy_replicon to drop the message
    // silently if the referenced entity is not yet in ServerEntityMap at deserialization
    // time (which happens when the entity is initially hidden from the client).
    // handle_ownership_granted performs the entity map lookup manually.
    app.add_server_message::<OwnershipGranted>(Channel::Ordered);

    // Register local messages
    app.add_message::<HostInteractionOccurred>();
    app.add_message::<HostMovableMotionEvent>();
    app.add_message::<HostFloorGearDroppedEvent>();
    app.add_message::<HostFloorGearPickedUpEvent>();

    // Register replicated components
    app.replicate::<TmxEntityId>();
    app.replicate::<Owner>();
    app.replicate::<Position>();
    app.replicate::<PlayerSprite>();
    app.replicate::<Stamina>();
    app.replicate::<PlayerGear>();
    app.replicate::<HeldObject>();
    app.replicate::<Hiding>();
    app.replicate::<PlayerSpectating>();
    app.replicate::<GearMarker>();
    app.replicate::<GearKind>();

    // Host/offline: spawn and tag player entities when InGame starts.
    // Gated by AuthorityRole so it runs on Host and Dedicated Server.
    app.add_systems(
        OnEnter(AppState::InGame),
        setup_mission_players.run_if(resource_exists::<AuthorityRole>),
    );

    // Server-side: message handlers + net state sync.
    app.add_systems(
        Update,
        (
            handle_interaction_request,
            broadcast_host_interactions,
            broadcast_movable_motion,
            handle_export_state,
            handle_request_pickup_gear,
            handle_ownership_released,
            handle_export_gear_state,
        )
            .run_if(resource_exists::<AuthorityRole>),
    );

    // Server-side: spawn player entities for clients that joined after mission start.
    app.add_systems(
        Update,
        spawn_late_joining_players
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(AppState::InGame))
            .run_if(resource_exists::<RepliconPlayerSpawningActive>),
    );

    // Server-side: truck loadout message handler (Truck phase only)
    app.add_systems(
        Update,
        handle_truck_loadout_message
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(GameState::Truck)),
    );

    // All local players (offline, host, join): send own state to the authority every frame.
    app.add_systems(
        Update,
        (send_export_state, send_export_gear_state)
            .run_if(in_state(AppState::InGame))
            .run_if(resource_exists::<LocalPlayerRole>),
    );

    // Client-side: apply replicated player state to local components
    app.add_systems(
        Update,
        (
            apply_remote_interaction,
            handle_ownership_granted,
            fallback_player_ownership_from_uuid,
        )
            .run_if(in_state(AppState::InGame))
            .run_if(is_pure_client),
    );

    // Debug system for player entities
    app.add_systems(
        Update,
        debug_player_entities.run_if(in_state(AppState::InGame)),
    );

    // Cleanup the spawning-active marker when leaving InGame
    app.add_systems(OnExit(AppState::InGame), cleanup_mission_players);

    // Client: hydrate gear entities that arrive via replication.
    app.add_systems(
        Update,
        hydrate_gear_system
            .run_if(is_pure_client)
            .run_if(in_state(AppState::InGame)),
    );
}

/// Client: fires when a GearKind component appears on an entity without GearHydrated.
/// Applies all type-specific components via the gear builder registry.
/// Gated to pure clients — authority nodes already have all components from gear_registry.spawn().
///
/// FIXME WARNING: The gear builder inserts components at their DEFAULT values
/// (e.g. Flashlight { status: Off }, Battery { level: 1.0 }, etc.).
/// The server-side gear may already be in a different state (battery drained, flashlight on, etc.).
/// Late-joining clients will see remote players' gear in its initial state, not the current state.
/// Gear state synchronisation is a separate follow-up task.
fn hydrate_gear_system(
    mut commands: Commands,
    gear_registry: Res<GearSpawnerRegistry>,
    q_added: Query<(Entity, &GearKind), (With<GearMarker>, Without<GearHydrated>)>,
) {
    for (entity, kind) in q_added.iter() {
        gear_registry.hydrate(&mut commands, entity, *kind);
        commands.entity(entity).insert(GearHydrated);
        info!(
            "hydrate_gear_system: hydrated gear entity {:?} kind={:?}",
            entity, kind
        );
    }
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

/// Server: called once on `OnEnter(AppState::InGame)`.
fn setup_mission_players(
    q_lobby: Query<&LobbyInfo>,
    q_spawn_points: Query<&Position, (With<PlayerSpawnPoint>, Without<PlayerSprite>)>,
    mut commands: Commands,
    gear_registry: Res<GearSpawnerRegistry>,
    difficulty: Res<CurrentDifficulty>,
) {
    // Signal that replicon-based player spawning is now active.
    // This marker causes spawn_joined_player (being deleted in Step 3) to be a no-op
    // on any node that still has the old system registered during the transition period.
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

        // --- Gear Initialization ---
        let mut player_gear = PlayerGear::default();
        let mut gear_id_counter = (net_id.0 % 1_000_000) * 1000;

        if difficulty.0.player_gear.left_hand.is_some() {
            let gear_entity =
                gear_registry.spawn(&mut commands, difficulty.0.player_gear.left_hand);
            player_gear.left_hand = Some(gear_entity);
            commands
                .entity(gear_entity)
                .insert((NetworkId(gear_id_counter), Replicated));
            gear_id_counter += 1;
        }
        if difficulty.0.player_gear.right_hand.is_some() {
            let gear_entity =
                gear_registry.spawn(&mut commands, difficulty.0.player_gear.right_hand);
            player_gear.right_hand = Some(gear_entity);
            commands
                .entity(gear_entity)
                .insert((NetworkId(gear_id_counter), Replicated));
            gear_id_counter += 1;
        }
        for kind in &difficulty.0.player_gear.inventory {
            if kind.is_some() {
                let gear_entity = gear_registry.spawn(&mut commands, *kind);
                player_gear.inventory.push(gear_entity);
                commands
                    .entity(gear_entity)
                    .insert((NetworkId(gear_id_counter), Replicated));
                gear_id_counter += 1;
            }
        }
        // Spawn the player skeleton. Every node (host, join client, dedicated server)
        // that replicates will receive this entity. Visual components are NOT added
        // here — hydrate_players_system handles that.
        let entity = commands
            .spawn((
                spawn_pos,
                unspatial_core::lerp_position::LerpPosition::new(spawn_pos),
                PlayerSprite::new(player.player_uuid, net_id, spawn_pos),
                net_id,
                Stamina::default(),
                player_gear,
                unspatial_core::direction::Direction::new_right(),
                unbehavior::components::Movable,
                unnavigation_core::components::waypoint::WaypointQueue::default(),
                unspatial_core::boardposition::MapEntityFieldBPos(spawn_pos.to_board_position()),
                untags_core::tags::PlayerTag,
                unrender_std::resources::visibility_data::VisibilityData::default(),
                unplayer_core::components::PlayerInput::default(),
                Replicated,
            ))
            .id();

        let is_host = player.current_socket.is_none();

        if is_host {
            // Host player: authority owns it locally. Insert LocallyOwned directly —
            // the OwnershipGranted message path is not used because the host is not
            // a pure client.
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

            // NOTE: We intentionally do NOT call visibility.set(..., false) here.
            // Hiding the entity from the owning client would prevent it from ever
            // arriving via replication, which makes OwnershipGranted impossible to
            // process on the client side (the entity would not be in ServerEntityMap).
            // Instead, the entity replicates to the owning client normally. The client
            // takes ownership via handle_ownership_granted or fallback_player_ownership_from_uuid
            // and removes Replicated so the server stops sending future position updates.

            // Notify the client of which entity it owns. On the client side,
            // handle_ownership_granted receives this message and inserts LocallyOwned
            // onto the entity — without this, the client never knows which entity
            // belongs to it, and send_export_state (which queries With<LocallyOwned>)
            // will never fire for that client.
            let client_id = from_owner_id(socket_owner_id);
            commands.write_message(ToClients {
                mode: SendMode::Direct(client_id),
                message: OwnershipGranted { entity },
            });
            info!(
                "setup_mission_players: spawned remote skeleton {:?} for player {} (owner={:?})",
                entity, player.player_uuid, socket_owner_id
            );
        }
    }
}

/// Server: on exit from `AppState::InGame`, remove the spawning-active marker.
fn cleanup_mission_players(mut commands: Commands) {
    commands.remove_resource::<RepliconPlayerSpawningActive>();
}

/// Server: runs every frame during InGame to spawn player entities for clients that
/// connected (and were added to lobby.players) after setup_mission_players already ran.
fn spawn_late_joining_players(
    q_lobby: Query<&LobbyInfo>,
    q_existing_sprites: Query<&PlayerSprite>,
    q_spawn_points: Query<&Position, With<unboard_core::components::spawning::PlayerSpawnPoint>>,
    mut commands: Commands,
    gear_registry: Res<GearSpawnerRegistry>,
    difficulty: Res<CurrentDifficulty>,
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

        let spawn_pos = spawn_points
            .get(idx % spawn_points.len().max(1))
            .copied()
            .unwrap_or(default_pos);

        let net_id = NetworkId::from(player.player_uuid);

        let mut player_gear = PlayerGear::default();
        let mut gear_id_counter = (net_id.0 % 1_000_000) * 1000;

        if difficulty.0.player_gear.left_hand.is_some() {
            let gear_entity =
                gear_registry.spawn(&mut commands, difficulty.0.player_gear.left_hand);
            player_gear.left_hand = Some(gear_entity);
            commands
                .entity(gear_entity)
                .insert((NetworkId(gear_id_counter), Replicated));
            gear_id_counter += 1;
        }
        if difficulty.0.player_gear.right_hand.is_some() {
            let gear_entity =
                gear_registry.spawn(&mut commands, difficulty.0.player_gear.right_hand);
            player_gear.right_hand = Some(gear_entity);
            commands
                .entity(gear_entity)
                .insert((NetworkId(gear_id_counter), Replicated));
            gear_id_counter += 1;
        }
        for kind in &difficulty.0.player_gear.inventory {
            if kind.is_some() {
                let gear_entity = gear_registry.spawn(&mut commands, *kind);
                player_gear.inventory.push(gear_entity);
                commands
                    .entity(gear_entity)
                    .insert((NetworkId(gear_id_counter), Replicated));
                gear_id_counter += 1;
            }
        }

        let socket_owner_id = player.current_socket.unwrap();
        let entity = commands
            .spawn((
                spawn_pos,
                unspatial_core::lerp_position::LerpPosition::new(spawn_pos),
                PlayerSprite::new(player.player_uuid, net_id, spawn_pos),
                net_id,
                Stamina::default(),
                player_gear,
                unspatial_core::direction::Direction::new_right(),
                unbehavior::components::Movable,
                unnavigation_core::components::waypoint::WaypointQueue::default(),
                unspatial_core::boardposition::MapEntityFieldBPos(spawn_pos.to_board_position()),
                untags_core::tags::PlayerTag,
                unrender_std::resources::visibility_data::VisibilityData::default(),
                unplayer_core::components::PlayerInput::default(),
                Owner(socket_owner_id),
                Replicated,
            ))
            .id();

        let client_id = from_owner_id(socket_owner_id);
        commands.write_message(ToClients {
            mode: SendMode::Direct(client_id),
            message: OwnershipGranted { entity },
        });
        info!(
            "spawn_late_joining_players: spawned skeleton {:?} for late-joining player {} (owner={:?})",
            entity, player.player_uuid, socket_owner_id
        );
    }
}

fn send_export_gear_state(
    q_local_gear: Query<
        (
            Entity,
            Option<&ungearitems_core::components::flashlight::Flashlight>,
        ),
        With<LocallyOwned>,
    >,
    mut writer: MessageWriter<ExportGearStateMessage>,
) {
    for (entity, flashlight) in q_local_gear.iter() {
        if let Some(flashlight) = flashlight {
            writer.write(ExportGearStateMessage {
                entity,
                is_on: flashlight.status
                    != ungearitems_core::components::flashlight::FlashlightStatus::Off,
                battery: 100.0, // TODO
                temperature: flashlight.inner_temp,
            });
        }
    }
}

/// Server: handle `ExportGearStateMessage` from connected clients.
fn handle_export_gear_state(
    mut reader: MessageReader<FromClient<ExportGearStateMessage>>,
    mut q_gear: Query<
        (
            &Owner,
            Option<&mut ungearitems_core::components::flashlight::Flashlight>,
        ),
        Without<LocallyOwned>,
    >,
) {
    for msg in reader.read() {
        if let Ok((owner, flashlight)) = q_gear.get_mut(msg.message.entity) {
            if from_owner_id(owner.0) != msg.client_id {
                continue;
            }
            if let Some(mut flashlight) = flashlight {
                flashlight.status = if msg.message.is_on {
                    ungearitems_core::components::flashlight::FlashlightStatus::High
                } else {
                    ungearitems_core::components::flashlight::FlashlightStatus::Off
                };
                flashlight.inner_temp = msg.message.temperature;
            }
        }
    }
}

/// Server: handle `ExportStateMessage` from connected clients.
fn handle_export_state(
    mut reader: MessageReader<FromClient<ExportStateMessage>>,
    mut q_players: Query<
        (
            &Owner,
            &mut Position,
            &mut Stamina,
            &mut PlayerSprite,
            Option<&mut PlayerSpectating>,
        ),
        Without<LocallyOwned>,
    >,
    mut commands: Commands,
) {
    for msg in reader.read() {
        for (owner, mut pos, mut stamina, mut sprite, spectating) in q_players.iter_mut() {
            if from_owner_id(owner.0) != msg.client_id {
                continue;
            }

            pos.x = msg.message.x;
            pos.y = msg.message.y;
            pos.z = msg.message.z;

            stamina.running = msg.message.is_running;
            stamina.current = msg.message.stamina * stamina.max;

            sprite.health = msg.message.health;
            sprite.sanity = msg.message.sanity;

            if msg.message.is_spectating
                && spectating.is_none()
                && let OwnerId::Client(e) = owner.0
            {
                commands.entity(e).insert(PlayerSpectating);
            }

            break;
        }
    }
}

/// Server: handle `InteractionRequestMessage` from connected clients.
fn handle_interaction_request(
    mut reader: MessageReader<FromClient<InteractionRequestMessage>>,
    q_interactive: Query<(Entity, &MapEntityFieldBPos)>,
    mut ev_interact: MessageWriter<ExecuteInteractionEvent>,
    mut ev_broadcast: MessageWriter<ToClients<RemoteInteractionBroadcast>>,
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
            ev_interact.write(ExecuteInteractionEvent {
                entity,
                ietype: msg.message.ietype.clone(),
                force_tuid: msg.message.force_tuid,
            });

            ev_broadcast.write(ToClients {
                mode: SendMode::BroadcastExcept(msg.client_id),
                message: RemoteInteractionBroadcast {
                    position: msg.message.position,
                    ietype: msg.message.ietype.clone(),
                    force_tuid: msg.message.force_tuid,
                },
            });
        }
    }
}

/// Server: broadcast host interactions.
fn broadcast_host_interactions(
    mut reader: MessageReader<HostInteractionOccurred>,
    mut ev_broadcast: MessageWriter<ToClients<RemoteInteractionBroadcast>>,
) {
    for msg in reader.read() {
        ev_broadcast.write(ToClients {
            mode: SendMode::Broadcast,
            message: RemoteInteractionBroadcast {
                position: msg.position,
                ietype: msg.ietype.clone(),
                force_tuid: msg.force_tuid,
            },
        });
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
                map_bpos: msg.map_bpos,
                start: msg.start,
                end: msg.end,
                duration: msg.duration,
                ease: msg.ease,
            },
        });
    }
}

fn handle_request_pickup_gear(
    mut reader: MessageReader<FromClient<RequestPickupGear>>,
    mut commands: Commands,
    q_gear: Query<(Entity, Option<&Owner>), With<FloorItemCollidable>>,
    filter_bit: Res<crate::plugin::GlobalFilterBit>,
    mut q_clients: Query<&mut ClientVisibility>,
) {
    for msg in reader.read() {
        let client_id = msg.client_id;
        let gear_entity = msg.message.entity;

        if let Ok((entity, owner)) = q_gear.get(gear_entity)
            && owner.is_none()
        {
            let owner_id = to_owner_id(client_id);
            // Grant ownership
            commands.entity(entity).insert(Owner(owner_id));

            // Pillar 5 Orphan step
            // commands.entity(entity).remove::<Replicated>(); // REVERTED: Server must keep Replicated

            if let OwnerId::Client(client_entity) = owner_id
                && let Ok(mut visibility) = q_clients.get_mut(client_entity)
            {
                visibility.set(entity, filter_bit.0, false);
            }

            commands.write_message(ToClients {
                mode: SendMode::Direct(client_id),
                message: OwnershipGranted { entity },
            });
        }
    }
}

fn handle_ownership_released(
    mut reader: MessageReader<FromClient<OwnershipReleased>>,
    mut commands: Commands,
    q_gear: Query<(Entity, &Owner)>,
    filter_bit: Res<crate::plugin::GlobalFilterBit>,
    mut q_clients: Query<&mut ClientVisibility>,
) {
    for msg in reader.read() {
        let client_id = msg.client_id;
        let gear_entity = msg.message.entity;

        if let Ok((entity, owner)) = q_gear.get(gear_entity)
            && from_owner_id(owner.0) == client_id
        {
            commands.entity(entity).remove::<Owner>();

            // Pillar 5 Re-Adopt step (Server side)
            commands.entity(entity).insert(Replicated);

            if let ClientId::Client(client_entity) = client_id
                && let Ok(mut visibility) = q_clients.get_mut(client_entity)
            {
                visibility.set(entity, filter_bit.0, true);
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

        let Some(mut p_gear) = q_players.iter_mut().find_map(|(owner, gear)| {
            if from_owner_id(owner.0) == sender_id {
                Some(gear)
            } else {
                None
            }
        }) else {
            continue;
        };

        match msg.message.action {
            TruckLoadoutAction::AddGear(kind) => {
                let has_space = p_gear.left_hand.is_none()
                    || p_gear.right_hand.is_none()
                    || p_gear.inventory.len() < 2;
                if !has_space {
                    continue;
                }
                let entity = gear_registry.spawn(&mut commands, kind);
                commands.entity(entity).insert(Replicated);
                if p_gear.left_hand.is_none() {
                    p_gear.left_hand = Some(entity);
                } else if p_gear.right_hand.is_none() {
                    p_gear.right_hand = Some(entity);
                } else {
                    p_gear.inventory.push(entity);
                }
            }
            TruckLoadoutAction::ClearHand(hand) => {
                let entity = match hand {
                    Hand::Left => p_gear.left_hand.take(),
                    Hand::Right => p_gear.right_hand.take(),
                };
                if let Some(e) = entity {
                    commands.entity(e).despawn();
                }
            }
            TruckLoadoutAction::ClearInventorySlot(idx) => {
                if idx < p_gear.inventory.len() {
                    let e = p_gear.inventory.remove(idx);
                    commands.entity(e).despawn();
                }
            }
        }
    }
}

fn apply_remote_interaction(
    mut reader: MessageReader<RemoteInteractionBroadcast>,
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
        }
    }
}

/// Client: Send own state to server.
fn send_export_state(
    q_local: Query<
        (
            &Position,
            &PlayerSprite,
            &Stamina,
            Has<Hiding>,
            Has<PlayerSpectating>,
        ),
        With<LocallyOwned>,
    >,
    mut writer: MessageWriter<ExportStateMessage>,
) {
    for (pos, sprite, stamina, is_hiding, is_spectating) in q_local.iter() {
        writer.write(ExportStateMessage {
            x: pos.x,
            y: pos.y,
            z: pos.z,
            is_running: stamina.running,
            frame: 0,
            is_hiding,
            stamina: stamina.percentage(),
            health: sprite.health,
            sanity: sprite.sanity,
            is_spectating,
        });
    }
}

/// Client: Fallback system that grants `LocallyOwned` on the player entity identified by UUID.
/// This handles any residual timing edge-case where `OwnershipGranted` arrives before the entity
/// appears in `ServerEntityMap`. Runs once per frame in InGame on pure clients until the entity
/// acquires `LocallyOwned`, at which point it falls out of the query and becomes a no-op.
fn fallback_player_ownership_from_uuid(
    q: Query<(Entity, &PlayerSprite), Without<LocallyOwned>>,
    local_player: Res<LocalPlayer>,
    mut commands: Commands,
    time: Res<Time>,
    mut log_timer: Local<f32>,
) {
    let Some(local_uuid) = local_player.0 else {
        // LocalPlayer not set yet — this is expected before identity is established.
        return;
    };

    // Periodic diagnostic log so we can tell what entities exist on this client.
    *log_timer -= time.delta_secs();
    if *log_timer <= 0.0 {
        *log_timer = 3.0;
        let candidates: Vec<_> = q.iter().map(|(e, s)| (e, s.id)).collect();
        if candidates.is_empty() {
            debug!(
                "fallback_player_ownership_from_uuid: looking for UUID={}, no PlayerSprite entities without LocallyOwned exist yet",
                local_uuid
            );
        } else {
            debug!(
                "fallback_player_ownership_from_uuid: looking for UUID={}, candidates={:?}",
                local_uuid, candidates
            );
        }
    }

    for (entity, sprite) in q.iter() {
        if sprite.id == local_uuid {
            info!(
                "fallback_player_ownership_from_uuid: granting LocallyOwned to {:?} via UUID match (UUID={})",
                entity, local_uuid
            );
            commands.entity(entity).insert(LocallyOwned);
            commands.entity(entity).remove::<Replicated>();
            // NOTE: We intentionally do NOT remove ConfirmHistory here.
            //
            // ConfirmHistory is bevy_replicon's per-entity history buffer used to decode
            // "mutate" (delta) messages: the server sends diffs relative to a confirmed
            // baseline, and ConfirmHistory holds that baseline on the client.
            //
            // When we remove Replicated and take local ownership, the server doesn't know
            // yet — it keeps sending mutate messages for ~1 RTT until our OwnershipReleased
            // message arrives. Those in-flight mutate packets still need ConfirmHistory to
            // decode. If we remove it here, every one of them errors with
            // "missing history component inserted on the first update message".
            //
            // ConfirmHistory should ideally be removed only after the server acknowledges
            // the transfer and stops sending updates — but bevy_replicon 0.39 has no
            // callback for that. Leaving it in place is the safe approach: it's a small
            // allocation and becomes unreachable once Replicated is gone.
            // .remove::<bevy_replicon::client::confirm_history::ConfirmHistory>();
        }
    }
}

/// Client: Handle ownership granted.
fn handle_ownership_granted(
    mut reader: MessageReader<OwnershipGranted>,
    entity_map: Res<ServerEntityMap>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let server_entity = msg.entity;
        if let Some(&client_entity) = entity_map.to_client().get(&server_entity) {
            info!("handle_ownership_granted: entity {:?}", client_entity);
            commands
                .entity(client_entity)
                .insert(LocallyOwned)
                .remove::<Replicated>();
            // FIXME: No idea why we need to remove that Confirm history, it causes tons of errors: unable to apply mutate message for tick `RepliconTick(250)`: `2416v0` missing history component inserted on the first update message.
            // .remove::<bevy_replicon::client::confirm_history::ConfirmHistory>()

            // FIXME: Pillar 5 Orphan step (Client side):
            // Remove from ServerEntityMap so replicon stops updating it.
            // User says remove_by_server exists but it is not in the public API of 0.38.2.
            // Mapping removal is currently skipped due to private API constraints.
            // Replicated removal should mitigate some issues, but this is technically broken.
        }
    }
}

/// Debug system for player entities.
fn debug_player_entities(
    q: Query<(
        Entity,
        &Owner,
        Has<LocallyOwned>,
        Has<Replicated>,
        Has<MainPlayer>,
        Has<Viewer>,
    )>,
    q_vis: Query<(
        Has<VisibilityData>,
        Has<Visibility>,
        Has<InheritedVisibility>,
        Has<ViewVisibility>,
    )>,
    q_state: Query<(
        Has<Position>,
        Has<unspatial_core::direction::Direction>,
        Has<PlayerSprite>,
        Has<unrender_std::components::animation::AnimationTimer>,
        Has<ungear_core::components::playergear::PlayerGear>,
        Has<unplayer_core::components::PlayerInput>,
        Has<Stamina>,
    )>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer -= time.delta_secs();
    if *timer > 0.0 {
        return;
    }
    *timer = 2.0;

    for (entity, owner, is_local, is_replicated, is_mainplayer, is_viewer) in q.iter() {
        let (has_visibility_data, has_visibility, has_inherited_visibility, has_view_visibility) =
            q_vis.get(entity).unwrap_or_default();
        let (has_pos, has_dir, has_sprite, has_anim, has_gear, has_input, has_stamina) =
            q_state.get(entity).unwrap_or_default();

        debug!(
            "PLAYER: Entity: {:?}, Owner: {:?}, LocallyOwned: {}, Replicated: {}, Main Player: {}, Viewer: {}, data: {has_visibility_data}, pos: {has_pos}, dir: {has_dir}, spr: {has_sprite}, anim: {has_anim}, gear: {has_gear}, inp: {has_input}, stam: {has_stamina}, vis: {has_visibility}, inh_vis: {has_inherited_visibility}, view_vis: {has_view_visibility}",
            entity, owner.0, is_local, is_replicated, is_mainplayer, is_viewer
        );
    }
}
