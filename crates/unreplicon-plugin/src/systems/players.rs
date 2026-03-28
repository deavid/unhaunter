mod replication;

use bevy::prelude::*;
use bevy_replicon::bytes::Bytes;
use bevy_replicon::prelude::{
    Channel, ClientId, ClientMessageAppExt, FromClient, Replicated, SendMode, ServerMessageAppExt,
    ToClients,
};
use bevy_replicon::shared::replication::deferred_entity::DeferredEntity;
use bevy_replicon::shared::replication::registry::ctx::{RemoveCtx, WriteCtx};
use bevy_replicon::shared::replication::registry::rule_fns::RuleFns;
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use unbehavior_core::behavior::Behavior;
use unbehavior_core::behavior::Interactive;
use unbehavior_core::components::FloorItemCollidable;
use unboard_core::components::spawning::PlayerSpawnPoint;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::{GearHydrated, GearMarker, GearSpawnerRegistry};
use ungear_core::types::gear::equipment::{EquipmentPosition, Hand};
use ungear_core::types::gear::kind::GearKind;
use ungearitems_core::components::flashlight::FlashlightStatus;
use uninteraction_core::events::InteractionRequestMessage;
use uninteraction_core::interaction::{ExecuteInteractionEvent, Toggleable};
use unlocomotion_core::components::PlayerLocomotionState;
use unplayer_core::components::{Hiding, PlayerSpectating, PlayerSprite};
use unreplicon_core::components::{LobbyInfo, RepliconPlayerSpawningActive};
use unreplicon_core::messages::{
    ExportGearStateMessage, ExportPlayerGearMessage, ExportStateMessage, FloorGearDespawnBroadcast,
    FloorGearSpawnBroadcast, GearSkeletonState, HostMovableMotionEvent, MovableMotionBroadcast,
    OwnershipGranted, RequestDrop, RequestGrab, SaltDroppedMessage, TruckLoadoutAction,
    TruckLoadoutMessage,
};
use unreplicon_core::network_id::NetworkId;
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unreplicon_core::resources::LocalPlayer;
use unspatial_core::boardposition::{BoardPosition, MapEntityFieldBPos};
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;
use untypes_core::roles::is_pure_client;
use untypes_core::roles::{AuthorityRole, LocalPlayerRole};
use untypes_core::states::{AppState, SimulationState};
use unvitals_core::components::{PlayerVitals, Stamina};

pub(super) fn app_setup(app: &mut App) {
    // Register client → server messages
    app.add_client_message::<ExportStateMessage>(Channel::Unreliable);
    app.add_client_message::<InteractionRequestMessage>(Channel::Ordered);
    app.add_client_message::<TruckLoadoutMessage>(Channel::Ordered);
    app.add_client_message::<SaltDroppedMessage>(Channel::Ordered);
    app.add_mapped_client_message::<RequestGrab>(Channel::Ordered);
    app.add_mapped_client_message::<RequestDrop>(Channel::Ordered);
    app.add_mapped_client_message::<ExportGearStateMessage>(Channel::Unreliable);
    app.add_mapped_client_message::<ExportPlayerGearMessage>(Channel::Unreliable);
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

    replication::app_setup(app);

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
            handle_export_state,
            handle_export_player_gear_state,
            handle_request_grab,
            handle_request_drop,
            handle_export_gear_state,
            handle_salt_drop,
        )
            .run_if(resource_exists::<AuthorityRole>),
    );

    // Server-side: spawn player entities for clients that joined after mission start.
    app.add_systems(
        Update,
        spawn_late_joining_players
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

    // All local players (offline, host, join): send own state to the authority every frame.
    app.add_systems(
        Update,
        send_export_state
            .run_if(in_state(AppState::InGame))
            .run_if(resource_exists::<LocalPlayerRole>),
    );
    // Gear state export only runs on pure join clients — the authority already has the
    // ground truth locally. Running it on host/offline would create a feedback loop where
    // send_export_gear_state sends the current state and handle_export_gear_state
    // re-applies a one-frame-stale copy, causing the flashlight to flicker.
    app.add_systems(
        Update,
        send_export_gear_state
            .in_set(ungearitems_core::GearStateExportSet)
            .run_if(in_state(AppState::InGame))
            .run_if(is_pure_client),
    );

    // Client-side: apply replicated player state to local components
    app.add_systems(
        Update,
        (
            handle_ownership_granted,
            fallback_player_ownership_from_uuid,
            propagate_gear_ownership,
            cleanup_gear_ownership,
        )
            .run_if(in_state(AppState::InGame))
            .run_if(is_pure_client),
    );

    // Cleanup the spawning-active marker when leaving InGame
    app.add_systems(
        OnEnter(SimulationState::TearingDown),
        cleanup_mission_players,
    );

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

        // Add skin components that are not in the registry hydrate path
        match kind {
            GearKind::SageBundle => {
                commands
                    .entity(entity)
                    .insert(ungearitems_core::components::sage::SageBundleSkin::new());
            }
            GearKind::QuartzStone => {
                commands
                    .entity(entity)
                    .insert(ungearitems_core::components::quartz::QuartzStoneSkin::default());
            }
            _ => {}
        }

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

/// Server: called once on `OnEnter(SimulationState::Spawning)`.
/// Spawns a minimal network skeleton for each lobby player.
/// Domain components (vitals, locomotion, gear, etc.) are inserted by each
/// domain's own hydration system reacting to `Added<PlayerSprite>`.
fn setup_mission_players(
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

        // Spawn the player skeleton. Domain plugins react to Added<PlayerSprite> to
        // insert their own components (vitals, locomotion, gear, navigation, etc.).
        let entity_commands = commands.spawn((
            spawn_pos,
            unspatial_core::lerp_position::LerpPosition::new(spawn_pos),
            PlayerSprite::new(player.player_uuid, net_id),
            net_id,
            unboard_core::resources::visibility_data::VisibilityData::default(),
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

            // NOTE: We intentionally do NOT call visibility.set(..., false) here.
            // Hiding the entity from the owning client would prevent it from ever
            // arriving via replication, which makes OwnershipGranted impossible to
            // process on the client side (the entity would not be in ServerEntityMap).
            let client_id = from_owner_id(socket_owner_id);
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
            info!(
                "setup_mission_players: spawned remote skeleton {:?} for player {} (owner={:?})",
                entity, player.player_uuid, socket_owner_id
            );
        }
    }
}

/// Server: on entering TearingDown, remove the spawning-active marker and despawn
/// all player skeletons (and their gear) so they cannot carry over into the next mission.
fn cleanup_mission_players(
    mut commands: Commands,
    q_players: Query<(Entity, &PlayerGear), With<PlayerSprite>>,
) {
    commands.remove_resource::<RepliconPlayerSpawningActive>();
    for (entity, gear) in q_players.iter() {
        for maybe_gear_entity in [gear.left_hand, gear.right_hand]
            .into_iter()
            .chain(gear.inventory.iter().copied().map(Some))
            .flatten()
        {
            if let Ok(mut ec) = commands.get_entity(maybe_gear_entity) {
                ec.despawn();
            }
        }
        commands.entity(entity).despawn();
    }
}

/// Server: runs every frame during InGame to spawn player entities for clients that
/// connected (and were added to lobby.players) after setup_mission_players already ran.
/// Domain plugins react to Added<PlayerSprite> to insert their own components.
fn spawn_late_joining_players(
    q_lobby: Query<&LobbyInfo>,
    q_existing_sprites: Query<&PlayerSprite>,
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

        let spawn_pos = spawn_points
            .get(idx % spawn_points.len().max(1))
            .copied()
            .unwrap_or(default_pos);

        let net_id = NetworkId::from(player.player_uuid);
        let socket_owner_id = player.current_socket.unwrap();

        let entity_commands = commands.spawn((
            spawn_pos,
            unspatial_core::lerp_position::LerpPosition::new(spawn_pos),
            PlayerSprite::new(player.player_uuid, net_id),
            net_id,
            unboard_core::resources::visibility_data::VisibilityData::default(),
            Owner(socket_owner_id),
            Replicated,
        ));
        let entity = entity_commands.id();

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

pub fn send_export_gear_state(
    q_local_player: Query<&PlayerGear, With<LocallyOwned>>,
    q_flashlight: Query<&ungearitems_core::components::flashlight::Flashlight>,
    q_uvtorch: Query<&ungearitems_core::components::uvtorch::UVTorch>,
    q_redtorch: Query<&ungearitems_core::components::redtorch::RedTorch>,
    q_repellent: Query<&ungearitems_core::components::repellentflask::RepellentFlask>,
    q_salt: Query<&ungearitems_core::components::salt::SaltData>,
    q_sage: Query<&ungearitems_core::components::sage::SageBundleData>,
    q_quartz: Query<&ungearitems_core::components::quartz::QuartzStoneData>,
    q_toggleable: Query<&Toggleable>,
    mut writer: MessageWriter<ExportGearStateMessage>,
) {
    for gear in q_local_player.iter() {
        for &entity in gear
            .left_hand
            .iter()
            .chain(gear.right_hand.iter())
            .chain(&gear.inventory)
        {
            let state = if let Ok(flashlight) = q_flashlight.get(entity) {
                GearSkeletonState::Flashlight(flashlight.status.clone())
            } else if let Ok(uvtorch) = q_uvtorch.get(entity) {
                GearSkeletonState::UVTorch(uvtorch.enabled)
            } else if let Ok(redtorch) = q_redtorch.get(entity) {
                GearSkeletonState::RedTorch(redtorch.enabled)
            } else if let Ok(repellent) = q_repellent.get(entity) {
                GearSkeletonState::RepellentFlask {
                    qty: repellent.qty,
                    liquid_content: repellent.liquid_content,
                }
            } else if let Ok(salt) = q_salt.get(entity) {
                GearSkeletonState::Salt(salt.charges)
            } else if let Ok(sage) = q_sage.get(entity) {
                GearSkeletonState::Sage {
                    is_active: sage.is_active,
                    consumed: sage.consumed,
                }
            } else if let Ok(quartz) = q_quartz.get(entity) {
                GearSkeletonState::Quartz(quartz.cracks)
            } else if let Ok(toggleable) = q_toggleable.get(entity) {
                GearSkeletonState::Toggleable(toggleable.is_on)
            } else {
                continue;
            };

            writer.write(ExportGearStateMessage { entity, state });
        }
    }
}

/// Server: handle `ExportGearStateMessage` from connected clients.
fn handle_export_gear_state(
    mut reader: MessageReader<FromClient<ExportGearStateMessage>>,
    mut q_gear: Query<
        (
            &Owner,
            &GearKind,
            Option<&mut ungearitems_core::components::flashlight::Flashlight>,
            Option<&mut ungearitems_core::components::uvtorch::UVTorch>,
            Option<&mut ungearitems_core::components::redtorch::RedTorch>,
            Option<&mut ungearitems_core::components::repellentflask::RepellentFlask>,
            Option<&mut ungearitems_core::components::salt::SaltData>,
            Option<&mut ungearitems_core::components::sage::SageBundleData>,
            Option<&mut ungearitems_core::components::quartz::QuartzStoneData>,
            Option<&mut Toggleable>,
        ),
        (Without<LocallyOwned>, With<Replicated>),
    >,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let state = msg.message.state.clone();

        trace!(
            "RECV: ExportGearStateMessage for entity {:?} from {:?}",
            entity, msg.client_id
        );

        let Ok((
            owner,
            gear_kind,
            flashlight,
            uvtorch,
            redtorch,
            repellent,
            salt,
            sage,
            quartz,
            toggleable,
        )) = q_gear.get_mut(entity)
        else {
            warn!(
                "RECV: ExportGearStateMessage: Entity {:?} not found or missing required components in q_gear query",
                entity
            );
            continue;
        };
        if from_owner_id(owner.0) != msg.client_id {
            warn!(
                "RECV: ExportGearStateMessage: Client ID mismatch. Expected {:?}, got {:?}",
                from_owner_id(owner.0),
                msg.client_id
            );
            continue;
        }

        // Handle different gear types based on GearKind
        match gear_kind {
            GearKind::Flashlight => {
                if let (Some(mut f), GearSkeletonState::Flashlight(status)) = (flashlight, state) {
                    if f.status != status {
                        info!(
                            "RECV: UPDATING FLASHLIGHT: Entity: {:?}, New Status: {:?}",
                            entity, status
                        );
                    }
                    let is_on = status != FlashlightStatus::Off;
                    f.status = status;
                    if let Some(mut t) = toggleable {
                        t.is_on = is_on;
                    }
                } else {
                    warn!(
                        "RECV: Failed to apply Flashlight state. Check components for entity {:?}",
                        entity
                    );
                }
            }
            GearKind::UVTorch => {
                if let (Some(mut uv), GearSkeletonState::UVTorch(enabled)) = (uvtorch, state) {
                    uv.enabled = enabled;
                    if let Some(mut t) = toggleable {
                        t.is_on = enabled;
                    }
                }
            }
            GearKind::RedTorch => {
                if let (Some(mut red), GearSkeletonState::RedTorch(enabled)) = (redtorch, state) {
                    red.enabled = enabled;
                    if let Some(mut t) = toggleable {
                        t.is_on = enabled;
                    }
                }
            }
            GearKind::RepellentFlask => {
                if let (
                    Some(mut r),
                    GearSkeletonState::RepellentFlask {
                        qty,
                        liquid_content,
                    },
                ) = (repellent, state)
                {
                    r.qty = qty;
                    r.liquid_content = liquid_content;
                    r.active = qty > 0 && liquid_content.is_some();
                }
            }
            GearKind::Salt => {
                if let (Some(mut s), GearSkeletonState::Salt(charges)) = (salt, state) {
                    s.charges = charges;
                }
            }
            GearKind::SageBundle => {
                if let (
                    Some(mut s),
                    GearSkeletonState::Sage {
                        is_active,
                        consumed,
                    },
                ) = (sage, state)
                {
                    s.is_active = is_active;
                    s.consumed = consumed;
                }
            }
            GearKind::QuartzStone => {
                if let (Some(mut q), GearSkeletonState::Quartz(cracks)) = (quartz, state) {
                    q.cracks = cracks;
                }
            }
            _ => {
                if let (Some(mut t), GearSkeletonState::Toggleable(is_on)) = (toggleable, state) {
                    t.is_on = is_on;
                }
            }
        }
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

/// Server: handle `ExportStateMessage` from connected clients.
fn handle_export_state(
    mut reader: MessageReader<FromClient<ExportStateMessage>>,
    mut q_players: Query<
        (
            Entity,
            &Owner,
            &mut Position,
            &mut Direction,
            &mut Stamina,
            &mut PlayerVitals,
            &mut PlayerLocomotionState,
            Option<&mut PlayerSpectating>,
        ),
        Without<LocallyOwned>,
    >,
    mut commands: Commands,
) {
    for msg in reader.read() {
        for (
            entity,
            owner,
            mut pos,
            mut dir,
            mut stamina,
            mut vitals,
            mut locomotion,
            spectating,
        ) in q_players.iter_mut()
        {
            if from_owner_id(owner.0) != msg.client_id {
                continue;
            }

            pos.x = msg.message.x;
            pos.y = msg.message.y;
            pos.z = msg.message.z;

            dir.dx = msg.message.direction_dx;
            dir.dy = msg.message.direction_dy;
            dir.dz = msg.message.direction_dz;

            stamina.running = msg.message.is_running;
            stamina.current = msg.message.stamina * stamina.max;

            if msg.message.is_hiding {
                commands.entity(entity).insert(Hiding { hiding_spot: None });
            } else {
                commands.entity(entity).remove::<Hiding>();
            }

            if msg.message.in_truck {
                commands
                    .entity(entity)
                    .insert(untruck_core::components::in_truck::InTruck);
            } else {
                commands
                    .entity(entity)
                    .remove::<untruck_core::components::in_truck::InTruck>();
            }

            vitals.health = msg.message.health;
            vitals.sanity = msg.message.sanity;
            locomotion.velocity.x = msg.message.movement_dx;
            locomotion.velocity.y = msg.message.movement_dy;

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

/// Server: handle `ExportPlayerGearMessage` from connected clients.
fn handle_export_player_gear_state(
    mut reader: MessageReader<FromClient<ExportPlayerGearMessage>>,
    mut q_players: Query<(&Owner, &mut PlayerGear), Without<LocallyOwned>>,
) {
    // TODO: Theoretical race condition: if an unreliable ExportPlayerGearMessage arrives out-of-order AFTER a RequestDrop has been processed, the server might briefly put the dropped item back into the player's inventory.
    for msg in reader.read() {
        for (owner, mut gear) in q_players.iter_mut() {
            if from_owner_id(owner.0) == msg.client_id {
                gear.left_hand = msg.message.left_hand;
                gear.right_hand = msg.message.right_hand;
                gear.inventory = msg.message.inventory.clone();
                gear.held_item = msg.message.held_item.clone();
                break;
            }
        }
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
                let rng_val = unfoundation_core::random_seed::heavy_rng_seed();
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

/// Client: Send own state to server.
fn send_export_state(
    q_local: Query<
        (
            &Position,
            &Direction,
            &PlayerVitals,
            &PlayerLocomotionState,
            &Stamina,
            &PlayerGear,
            Has<Hiding>,
            Has<untruck_core::components::in_truck::InTruck>,
            Has<PlayerSpectating>,
        ),
        With<LocallyOwned>,
    >,
    mut writer: MessageWriter<ExportStateMessage>,
    mut gear_writer: MessageWriter<ExportPlayerGearMessage>,
) {
    for (pos, dir, vitals, locomotion, stamina, gear, is_hiding, in_truck, is_spectating) in
        q_local.iter()
    {
        writer.write(ExportStateMessage {
            x: pos.x,
            y: pos.y,
            z: pos.z,
            direction_dx: dir.dx,
            direction_dy: dir.dy,
            direction_dz: dir.dz,
            is_running: stamina.running,
            frame: 0,
            is_hiding,
            in_truck,
            stamina: stamina.percentage(),
            health: vitals.health,
            sanity: vitals.sanity,
            movement_dx: locomotion.velocity.x,
            movement_dy: locomotion.velocity.y,
            is_spectating,
        });

        gear_writer.write(ExportPlayerGearMessage {
            left_hand: gear.left_hand,
            right_hand: gear.right_hand,
            inventory: gear.inventory.clone(),
            held_item: gear.held_item.clone(),
        });
    }
}

/// Client: Fallback system that grants `LocallyOwned` on the player entity identified by UUID.
/// This handles any residual timing edge-case where `OwnershipGranted` arrives before the entity
/// appears in `ServerEntityMap`. Runs once per frame in InGame on pure clients until the entity
/// acquires `LocallyOwned`, at which point it falls out of the query and becomes a no-op.
fn fallback_player_ownership_from_uuid(
    q: Query<(Entity, &PlayerSprite), Without<LocallyOwned>>,
    q_player_gear: Query<&PlayerGear>,
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

            // Also grant LocallyOwned to all gear entities this player holds
            if let Ok(gear) = q_player_gear.get(entity) {
                if let Some(left_hand) = gear.left_hand {
                    commands.entity(left_hand).insert(LocallyOwned);
                }
                if let Some(right_hand) = gear.right_hand {
                    commands.entity(right_hand).insert(LocallyOwned);
                }
                for &gear_entity in &gear.inventory {
                    commands.entity(gear_entity).insert(LocallyOwned);
                }
            }
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
    q_player_gear: Query<&PlayerGear>,
) {
    for msg in reader.read() {
        let server_entity = msg.entity;
        let client_entity = entity_map
            .to_client()
            .get(&server_entity)
            .copied()
            .unwrap_or(server_entity);
        info!("handle_ownership_granted: entity {:?}", client_entity);
        commands.entity(client_entity).insert(LocallyOwned);

        // Also grant LocallyOwned to all gear entities this player holds
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

/// Discards the server's value without writing it to the component.
fn noop_write<C: Component>(
    ctx: &mut WriteCtx,
    rule_fns: &RuleFns<C>,
    _entity: &mut DeferredEntity,
    message: &mut Bytes,
) -> Result<(), bevy::prelude::BevyError> {
    // We use the public deserialize and discard the result.
    // This advances the message cursor correctly.
    let _ = rule_fns.deserialize(ctx, message)?;
    Ok(())
}

/// Suppresses the server's component removal completely.
fn noop_remove(_ctx: &mut RemoveCtx, _entity: &mut DeferredEntity) {
    // Intentionally empty.
}
