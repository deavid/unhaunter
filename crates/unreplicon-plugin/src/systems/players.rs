use bevy::prelude::*;
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientId, ClientMessageAppExt, FromClient, Replicated,
    SendMode, ServerMessageAppExt, ToClients,
};
use bevy_replicon::server::visibility::client_visibility::ClientVisibility;
use unbehavior::components::FloorItemCollidable;
use unboard_core::components::spawning::PlayerSpawnPoint;
use unfoundation_core::types::gear::Hand;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use uninteraction_core::interaction::ExecuteInteractionEvent;
use untypes_core::roles::{AuthorityRole, LocalPlayerRole};
use unplayer_core::components::{Hiding, PlayerSpectating, PlayerSprite, Stamina};
use unreplicon_core::ownership::{Owner, OwnerId, LocallyOwned};
use unreplicon_core::components::{LobbyInfo, RepliconPlayerSpawningActive, SelectedMission};
use unreplicon_core::messages::{
    FloorGearDespawnBroadcast, FloorGearSpawnBroadcast, HostFloorGearDroppedEvent,
    HostFloorGearPickedUpEvent, HostInteractionOccurred, HostMovableMotionEvent,
    InteractionRequestMessage, MovableMotionBroadcast, PlayerMoveMessage,
    RemoteInteractionBroadcast, TruckLoadoutAction, TruckLoadoutMessage,
    ExportStateMessage, OwnershipGranted, RequestPickupGear, OwnershipReleased, ExportGearStateMessage
};
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use unspatial_core::boardposition::{BoardPosition, MapEntityFieldBPos};
use unspatial_core::position::Position;
use untypes_core::roles::is_pure_client;
use untypes_core::states::{AppState, GameState};
use ungear_core::components::playergear::HeldObject;

pub(super) fn app_setup(app: &mut App) {
    // Register client → server messages
    app.add_client_message::<ExportStateMessage>(Channel::Unreliable);
    app.add_client_message::<PlayerMoveMessage>(Channel::Unreliable);
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
    app.add_mapped_server_message::<OwnershipGranted>(Channel::Ordered);

    // Register local messages
    app.add_message::<HostInteractionOccurred>();
    app.add_message::<HostMovableMotionEvent>();
    app.add_message::<HostFloorGearDroppedEvent>();
    app.add_message::<HostFloorGearPickedUpEvent>();

    // Client: observe SelectedMission to enter MissionLoading
    app.add_observer(on_selected_mission_added);

    // Register replicated components
    app.replicate::<Owner>();
    app.replicate::<Position>();
    app.replicate::<PlayerSprite>();
    app.replicate::<Stamina>();
    app.replicate::<PlayerGear>();
    app.replicate::<HeldObject>();
    app.replicate::<Hiding>();
    app.replicate::<PlayerSpectating>();

    // Host/offline: spawn and tag player entities when InGame starts.
    // SP-6.3: gated by LocalPlayerRole so dedicated servers (no local player) never run this.
    app.add_systems(
        OnEnter(AppState::InGame),
        setup_mission_players.run_if(resource_exists::<LocalPlayerRole>),
    );

    // Server-side: message handlers + net state sync.
    app.add_systems(
        Update,
        (
            handle_player_move,
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
        )
            .run_if(in_state(AppState::InGame))
            .run_if(is_pure_client),
    );

    // Cleanup the spawning-active marker when leaving InGame
    app.add_systems(OnExit(AppState::InGame), cleanup_mission_players);
}


/// Helper: convert Replicon ClientId to OwnerId.
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
    q_host_player: Query<(Entity, &Position, &PlayerSprite)>,
    q_lobby: Query<&LobbyInfo>,
    q_spawn_points: Query<&Position, (With<PlayerSpawnPoint>, Without<PlayerSprite>)>,
    mut commands: Commands,
    filter_bit: Res<crate::plugin::GlobalFilterBit>,
    mut q_clients: Query<&mut ClientVisibility>,
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
    for (entity, _pos, _player_sprite) in q_host_player.iter() {
        commands.entity(entity).insert((
            Replicated,
            Owner(OwnerId::Server),
            LocallyOwned,
        ));

        info!(
            "setup_mission_players: host player entity {:?} marked Replicated, LocallyOwned and Hidden",
            entity
        );
    }

    // Spawn bare network entities for remote clients.
    let Ok(lobby) = q_lobby.single() else {
        warn!("setup_mission_players: no LobbyInfo entity found — skipping remote player spawn");
        return;
    };

    for (idx, player) in lobby.players.iter().enumerate() {
        let Some(socket_owner_id) = player.current_socket else {
            // Host handled above.
            continue;
        };
        let client_id = from_owner_id(socket_owner_id);

        // Pick a deterministic spawn point based on lobby order.
        let spawn_pos = spawn_points
            .get(idx % spawn_points.len().max(1))
            .copied()
            .unwrap_or(default_pos);
        let remote_entity = commands
            .spawn((
                Replicated,
                spawn_pos,
                unspatial_core::lerp_position::LerpPosition::new(spawn_pos),
                PlayerSprite::default(),
                Stamina::default(),
                PlayerGear::default(),
                Owner(socket_owner_id),
            ))
            .id();

        if let OwnerId::Client(client_entity) = socket_owner_id {
            if let Ok(mut visibility) = q_clients.get_mut(client_entity) {
                visibility.set(remote_entity, filter_bit.0, false);
            }
        }

        info!(
            "setup_mission_players: spawned replicated entity {:?} for client {:?}",
            remote_entity, client_id
        );

        // Notify client of ownership
        commands.write_message(ToClients {
            mode: SendMode::Direct(client_id),
            message: OwnershipGranted {
                entity: remote_entity,
            },
        });
    }
}

/// Server: on exit from `AppState::InGame`, remove the spawning-active marker.
fn cleanup_mission_players(mut commands: Commands) {
    commands.remove_resource::<RepliconPlayerSpawningActive>();
}

/// Server: handle `PlayerMoveMessage` from connected clients. (Legacy, still useful for now)
fn handle_player_move(
    mut reader: MessageReader<FromClient<PlayerMoveMessage>>,
    mut q_players: Query<(&Owner, &mut Position, &mut Stamina, &mut PlayerSprite)>,
) {
    for msg in reader.read() {
        for (owner, mut pos, mut stamina, mut sprite) in q_players.iter_mut() {
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

            break;
        }
    }
}

fn send_export_gear_state(
    q_local_gear: Query<(Entity, Option<&ungearitems_core::components::flashlight::Flashlight>), With<LocallyOwned>>,
    mut writer: MessageWriter<ExportGearStateMessage>,
) {
    for (entity, flashlight) in q_local_gear.iter() {
        if let Some(flashlight) = flashlight {
            writer.write(ExportGearStateMessage {
                entity,
                is_on: flashlight.status != ungearitems_core::components::flashlight::FlashlightStatus::Off,
                battery: 100.0, // TODO
                temperature: flashlight.inner_temp,
            });
        }
    }
}

fn handle_export_gear_state(
    mut reader: MessageReader<FromClient<ExportGearStateMessage>>,
    mut q_gear: Query<(&Owner, Option<&mut ungearitems_core::components::flashlight::Flashlight>)>,
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
    mut q_players: Query<(&Owner, &mut Position, &mut Stamina, &mut PlayerSprite, Option<&mut PlayerSpectating>)>,
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

            if msg.message.is_spectating && spectating.is_none() {
                if let OwnerId::Client(e) = owner.0 {
                    commands.entity(e).insert(PlayerSpectating);
                }
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

        if let Ok((entity, owner)) = q_gear.get(gear_entity) {
            if owner.is_none() {
                let owner_id = to_owner_id(client_id);
                // Grant ownership
                commands.entity(entity).insert(Owner(owner_id));

                // Pillar 5 Orphan step
                // commands.entity(entity).remove::<Replicated>(); // REVERTED: Server must keep Replicated

                if let OwnerId::Client(client_entity) = owner_id {
                    if let Ok(mut visibility) = q_clients.get_mut(client_entity) {
                        visibility.set(entity, filter_bit.0, false);
                    }
                }

                commands.write_message(ToClients {
                    mode: SendMode::Direct(client_id),
                    message: OwnershipGranted { entity },
                });
            }
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

        if let Ok((entity, owner)) = q_gear.get(gear_entity) {
            if from_owner_id(owner.0) == client_id {
                commands.entity(entity).remove::<Owner>();

                // Pillar 5 Re-Adopt step (Server side)
                commands.entity(entity).insert(Replicated);

                if let ClientId::Client(client_entity) = client_id {
                    if let Ok(mut visibility) = q_clients.get_mut(client_entity) {
                        visibility.set(entity, filter_bit.0, true);
                    }
                }
            }
        }
    }
}


fn on_selected_mission_added(
    _trigger: On<Add, SelectedMission>,
    local_player: Option<Res<LocalPlayerRole>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if local_player.is_some() {
        next_app_state.set(AppState::MissionLoading);
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
                .remove::<Replicated>()
                .remove::<bevy_replicon::client::confirm_history::ConfirmHistory>();

            // FIXME: Pillar 5 Orphan step (Client side):
            // Remove from ServerEntityMap so replicon stops updating it.
            // User says remove_by_server exists but it is not in the public API of 0.38.2.
            // Mapping removal is currently skipped due to private API constraints.
            // Replicated removal should mitigate some issues, but this is technically broken.
        }
    }
}
