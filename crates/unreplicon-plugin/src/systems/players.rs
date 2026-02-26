use bevy::prelude::*;
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientId, ClientMessageAppExt, ConnectedClient, FromClient, Replicated,
    SendMode, ServerMessageAppExt, ServerState, ToClients,
};
use bevy_replicon::shared::backend::connected_client::NetworkId as RepliconNetworkId;
use std::collections::HashMap;
use unbehavior::components::FloorItemCollidable;
use unboard_core::components::spawning::PlayerSpawnPoint;
use unfoundation_core::random_seed::heavy_rng_seed;
use unfoundation_core::types::gear::EquipmentPosition;
use unfoundation_core::types::gear::Hand;
use ungear_core::components::core::Battery;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use ungear_core::types::gear::kind::GearKind;
use ungearitems_core::components::emfmeter::EMFMeter;
use ungearitems_core::components::flashlight::Flashlight;
use ungearitems_core::components::repellentflask::RepellentFlask;
use ungearitems_core::components::sage::SageBundleData;
use ungearitems_core::components::spiritbox::SpiritBox;
use ungearitems_core::components::thermometer::Thermometer;
use uninteraction_core::interaction::{ExecuteInteractionEvent, Toggleable};
use unplayer_core::components::{Hiding, MainPlayer, PlayerSpectating, PlayerSprite, Stamina};
use unreplicon_core::components::{LobbyInfo, RepliconPlayerSpawningActive};
use unreplicon_core::messages::{
    FloorGearDespawnBroadcast, FloorGearSpawnBroadcast, HostFloorGearDroppedEvent,
    HostFloorGearPickedUpEvent, HostInteractionOccurred, HostMovableMotionEvent,
    InteractionRequestMessage, MovableMotionBroadcast, PlayerMoveMessage,
    RemoteInteractionBroadcast, TruckLoadoutAction, TruckLoadoutMessage,
};
use unreplicon_core::net_components::{
    EMFMeterNet, FlashlightNet, NetworkPosition, PlayerGearKindNet, PlayerNetInfo, PlayerStateNet,
    RepellentFlaskNet, SageBundleNet, SpiritBoxNet, ThermometerNet,
};
use unreplicon_core::network_id::NetworkId;
use unreplicon_core::resources::{FloorGearCache, FloorGearEntry};
use unspatial_core::boardposition::{BoardPosition, MapEntityFieldBPos};
use unspatial_core::components::NetworkOriginalMapPosition;
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;
use untypes_core::states::{AppState, GameState};

pub(super) fn app_setup(app: &mut App) {
    // Register client → server messages
    app.add_client_message::<PlayerMoveMessage>(Channel::Unreliable);
    app.add_client_message::<InteractionRequestMessage>(Channel::Ordered);
    app.add_client_message::<TruckLoadoutMessage>(Channel::Ordered);
    // Register server → client messages
    app.add_server_message::<RemoteInteractionBroadcast>(Channel::Ordered);
    app.add_server_message::<MovableMotionBroadcast>(Channel::Ordered);
    app.add_server_message::<FloorGearSpawnBroadcast>(Channel::Ordered);
    app.add_server_message::<FloorGearDespawnBroadcast>(Channel::Ordered);
    // Register local-only bridge events
    app.add_message::<HostInteractionOccurred>();
    app.add_message::<HostMovableMotionEvent>();
    app.add_message::<HostFloorGearDroppedEvent>();
    app.add_message::<HostFloorGearPickedUpEvent>();

    // Floor gear cache: tracks items on the floor for late-joining clients (C2)
    app.init_resource::<FloorGearCache>();
    app.add_systems(
        OnEnter(AppState::InGame),
        reset_floor_gear_cache.run_if(in_state(ServerState::Running)),
    );
    app.add_systems(
        OnExit(AppState::InGame),
        reset_floor_gear_cache.run_if(in_state(ServerState::Running)),
    );
    // Send existing floor gear to clients that connect mid-mission
    app.add_observer(send_floor_gear_to_new_client);

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
    app.replicate::<PlayerGearKindNet>();

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
            broadcast_host_interactions,
            broadcast_movable_motion,
            broadcast_floor_gear_drop,
            broadcast_floor_gear_pickup,
            sync_player_state_to_net,
            sync_held_object_to_net,
            sync_gear_to_net,
        )
            .run_if(in_state(ServerState::Running)),
    );

    // Server-side: truck loadout message handler (Truck phase only)
    app.add_systems(
        Update,
        handle_truck_loadout_message
            .run_if(in_state(ServerState::Running))
            .run_if(in_state(GameState::Truck)),
    );

    // Client-side: send local position to server (Join clients only)
    app.add_systems(
        Update,
        send_local_player_position
            .run_if(in_state(AppState::InGame))
            .run_if(not(in_state(ServerState::Running))),
    );

    // Client-side: apply replicated player state to local components
    app.add_systems(
        Update,
        (
            apply_player_state_net,
            apply_gear_kind_net,
            apply_flashlight_net,
            apply_thermometer_net,
            apply_emf_net,
            apply_spiritbox_net,
            apply_sage_net,
            apply_repellent_net,
            apply_remote_interaction,
            apply_floor_gear_spawn,
            apply_floor_gear_despawn,
            apply_held_object_position,
        )
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
            FlashlightNet::default(),
            ThermometerNet::default(),
            EMFMeterNet::default(),
            SpiritBoxNet::default(),
            SageBundleNet::default(),
            RepellentFlaskNet::default(),
            PlayerGearKindNet::default(),
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
                FlashlightNet::default(),
                ThermometerNet::default(),
                EMFMeterNet::default(),
                SpiritBoxNet::default(),
                SageBundleNet::default(),
                RepellentFlaskNet::default(),
                PlayerGearKindNet::default(),
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
/// `ExecuteInteractionEvent` on the server. Also broadcasts
/// `RemoteInteractionBroadcast` to all other join clients (excluding the
/// originator) so their local door/switch visual stays in sync.
fn handle_interaction_request(
    mut reader: MessageReader<FromClient<InteractionRequestMessage>>,
    q_interactive: Query<(Entity, &MapEntityFieldBPos)>,
    mut ev_interact: MessageWriter<ExecuteInteractionEvent>,
    mut ev_broadcast: MessageWriter<ToClients<RemoteInteractionBroadcast>>,
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

            // Broadcast to all join clients except the one who sent this
            // request (they already applied the interaction locally for
            // zero-latency feedback).
            ev_broadcast.write(ToClients {
                mode: SendMode::BroadcastExcept(msg.client_id),
                message: RemoteInteractionBroadcast {
                    position: msg.position,
                    ietype: msg.ietype.clone(),
                    force_tuid: msg.force_tuid,
                },
            });
        } else {
            warn!(
                "handle_interaction_request: no entity found at board position {:?}",
                target_bpos
            );
        }
    }
}

/// Server: when the host player interacts with an object (fired locally by
/// `player_interaction_system`), broadcast the change to all connected join
/// clients so their door/switch state stays in sync.
///
/// The host already applied the interaction locally (via `ExecuteInteractionEvent`)
/// before writing this event, so `SendMode::Broadcast` (which targets connected
/// clients only, not the server/host itself) avoids a double-toggle.
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

/// Server: broadcast a ghost-induced movable-object motion to all join clients
/// so they replay the same tween animation locally.
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

/// Server: relay `HostFloorGearDroppedEvent` to all join clients as a
/// `FloorGearSpawnBroadcast` so they can spawn a matching floor gear entity.
/// Also pushes an entry to [`FloorGearCache`] for late-joining clients.
fn broadcast_floor_gear_drop(
    mut reader: MessageReader<HostFloorGearDroppedEvent>,
    mut ev_broadcast: MessageWriter<ToClients<FloorGearSpawnBroadcast>>,
    mut cache: ResMut<FloorGearCache>,
) {
    for msg in reader.read() {
        cache.entries.push(FloorGearEntry {
            kind: msg.kind,
            pos: msg.pos,
            direction: msg.direction,
        });
        ev_broadcast.write(ToClients {
            mode: SendMode::Broadcast,
            message: FloorGearSpawnBroadcast {
                kind: msg.kind,
                pos: msg.pos,
                direction: msg.direction,
            },
        });
    }
}

/// Server: relay `HostFloorGearPickedUpEvent` to all join clients as a
/// `FloorGearDespawnBroadcast` so they can despawn their local floor gear entity.
/// Also removes the nearest matching entry from [`FloorGearCache`].
fn broadcast_floor_gear_pickup(
    mut reader: MessageReader<HostFloorGearPickedUpEvent>,
    mut ev_broadcast: MessageWriter<ToClients<FloorGearDespawnBroadcast>>,
    mut cache: ResMut<FloorGearCache>,
) {
    for msg in reader.read() {
        let pos = msg.pos;
        // Remove the nearest cache entry within 0.6 units of the pick-up position.
        let mut best_idx: Option<usize> = None;
        let mut best_dist = 0.6_f32;
        for (i, entry) in cache.entries.iter().enumerate() {
            let d = floor_gear_pos_dist(entry.pos, pos);
            if d < best_dist {
                best_dist = d;
                best_idx = Some(i);
            }
        }
        if let Some(idx) = best_idx {
            cache.entries.remove(idx);
        }
        ev_broadcast.write(ToClients {
            mode: SendMode::Broadcast,
            message: FloorGearDespawnBroadcast { pos },
        });
    }
}

/// Euclidean distance between two `[x, y, z]` positions.
fn floor_gear_pos_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Server: reset the [`FloorGearCache`] to empty.
///
/// Runs on `OnEnter(AppState::InGame)` and `OnExit(AppState::InGame)` so stale
/// entries never leak between missions.
fn reset_floor_gear_cache(mut commands: Commands) {
    commands.insert_resource(FloorGearCache::default());
}

/// Server observer: when a new client connects, replay all cached floor gear entries
/// directly to that client so they see items dropped before they joined.
fn send_floor_gear_to_new_client(
    trigger: On<Insert, ConnectedClient>,
    cache: Res<FloorGearCache>,
    mut ev_broadcast: MessageWriter<ToClients<FloorGearSpawnBroadcast>>,
) {
    if cache.entries.is_empty() {
        return;
    }
    let client_id = ClientId::Client(trigger.entity);
    for entry in &cache.entries {
        ev_broadcast.write(ToClients {
            mode: SendMode::Direct(client_id),
            message: FloorGearSpawnBroadcast {
                kind: entry.kind,
                pos: entry.pos,
                direction: entry.direction,
            },
        });
    }
}

/// Client: spawn a local floor gear entity when the server broadcasts a drop.
///
/// Calls `gear_registry.spawn()` for the given kind and inserts `FloorItemCollidable`,
/// `DeployedGear`, and `EquipmentPosition::Deployed` so that `update_deployed_gear_sprites`
/// in `ungear-plugin` will add the visual components automatically.
fn apply_floor_gear_spawn(
    mut reader: MessageReader<FloorGearSpawnBroadcast>,
    gear_registry: Res<GearSpawnerRegistry>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let entity = gear_registry.spawn(&mut commands, msg.kind);
        commands.entity(entity).insert((
            Position {
                x: msg.pos[0],
                y: msg.pos[1],
                z: msg.pos[2],
                visual_priority: 0.0,
            },
            FloorItemCollidable,
            EquipmentPosition::Deployed,
            DeployedGear {
                direction: Direction {
                    dx: msg.direction[0],
                    dy: msg.direction[1],
                    dz: msg.direction[2],
                },
            },
        ));
    }
}

/// Client: despawn the local floor gear entity nearest to the broadcasted position.
///
/// Finds the `FloorItemCollidable` entity within 0.6 units of `pos` and despawns it,
/// mirroring the server's `grab_object` remove of `FloorItemCollidable`.
fn apply_floor_gear_despawn(
    mut reader: MessageReader<FloorGearDespawnBroadcast>,
    q_floor_gear: Query<(Entity, &Position), With<FloorItemCollidable>>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let target = Position {
            x: msg.pos[0],
            y: msg.pos[1],
            z: msg.pos[2],
            visual_priority: 0.0,
        };
        let mut closest_entity = None;
        let mut min_dist = 0.6_f32;
        for (entity, pos) in q_floor_gear.iter() {
            let dist = pos.distance(&target);
            if dist < min_dist {
                min_dist = dist;
                closest_entity = Some(entity);
            }
        }
        if let Some(entity) = closest_entity {
            commands.entity(entity).despawn();
        }
    }
}

/// Server: read each player's `PlayerGear.held_item` and update
/// `PlayerStateNet.held_object_bpos` so remote clients can track the carried
/// world object.
///
/// Only non-gear map objects have `NetworkOriginalMapPosition` (gear entities are
/// local spawns). If the held entity lacks that component (e.g., it is a gear item)
/// the field is set to `None`.
fn sync_held_object_to_net(
    mut q_players: Query<(&PlayerGear, &mut PlayerStateNet)>,
    q_map_pos: Query<&NetworkOriginalMapPosition>,
) {
    for (gear, mut state_net) in q_players.iter_mut() {
        state_net.held_object_bpos = gear.held_item.as_ref().and_then(|held| {
            q_map_pos.get(held.entity).ok().map(|mp| {
                [
                    mp.position.x as i32,
                    mp.position.y as i32,
                    mp.position.z as i32,
                ]
            })
        });
    }
}

/// Client: every frame, move a carried world object to follow the remote player
/// that is holding it.
///
/// Reads `PlayerStateNet.held_object_bpos` for each non-`MainPlayer` entity.
/// When the field is `Some`, finds the map entity with that original board
/// position and sets its `Position` to the player's replicated `NetworkPosition`
/// with a +0.25 z-offset to represent the raised carrying height.
///
/// When the field is `None` the system does nothing — the object stays at
/// whatever position it was last set to (approximately the drop location).
fn apply_held_object_position(
    q_remote: Query<(Entity, &PlayerStateNet, &NetworkPosition), Without<MainPlayer>>,
    q_map_entities: Query<(Entity, &NetworkOriginalMapPosition)>,
    mut q_pos: Query<&mut Position>,
    mut held_cache: Local<HashMap<Entity, Entity>>,
) {
    for (player_entity, state_net, net_pos) in q_remote.iter() {
        if let Some(bpos_arr) = state_net.held_object_bpos {
            // Object is being carried: update carry position and remember the held entity.
            let target_bpos = BoardPosition {
                x: bpos_arr[0] as i64,
                y: bpos_arr[1] as i64,
                z: bpos_arr[2] as i64,
            };
            let found = q_map_entities
                .iter()
                .find(|(_, mp)| mp.position == target_bpos)
                .map(|(e, _)| e);
            if let Some(entity) = found
                && let Ok(mut pos) = q_pos.get_mut(entity)
            {
                held_cache.insert(player_entity, entity);
                pos.x = net_pos.x;
                pos.y = net_pos.y;
                pos.z = net_pos.z + 0.25;
            }
        } else {
            // Object was just dropped (Some → None): snap to floor level to clear the carry offset.
            if let Some(held_entity) = held_cache.remove(&player_entity)
                && let Ok(mut pos) = q_pos.get_mut(held_entity)
            {
                pos.x = net_pos.x;
                pos.y = net_pos.y;
                pos.z = net_pos.z;
            }
        }
    }
}

/// Server: handle `TruckLoadoutMessage` from join clients during the truck phase.
///
/// Applies the requested loadout action to the server's copy of the player's
/// `PlayerGear`. This ensures `sync_gear_to_net` will read the chosen gear at
/// mission start, so all clients see the correct gear state.
fn handle_truck_loadout_message(
    mut reader: MessageReader<FromClient<TruckLoadoutMessage>>,
    q_network_id: Query<Option<&RepliconNetworkId>>,
    mut q_players: Query<(&PlayerNetInfo, &mut PlayerGear)>,
    gear_registry: Res<GearSpawnerRegistry>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let sender_id = client_network_id(msg.client_id, &q_network_id);

        let Some(mut p_gear) = q_players.iter_mut().find_map(|(net_info, gear)| {
            if net_info.client_id == sender_id {
                Some(gear)
            } else {
                None
            }
        }) else {
            warn!(
                "handle_truck_loadout_message: no player entity found for client {}",
                sender_id
            );
            continue;
        };

        match &msg.message.action {
            TruckLoadoutAction::AddGear(kind) => {
                if kind.is_none() {
                    continue;
                }
                let has_space = p_gear.left_hand.is_none()
                    || p_gear.right_hand.is_none()
                    || p_gear.inventory.len() < 2;
                if !has_space {
                    continue;
                }
                let entity = gear_registry.spawn(&mut commands, *kind);
                commands
                    .entity(entity)
                    .insert(NetworkId(heavy_rng_seed().max(1000)));
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
                if *idx < p_gear.inventory.len() {
                    let e = p_gear.inventory.remove(*idx);
                    commands.entity(e).despawn();
                }
            }
        }
    }
}

/// Client: receive `RemoteInteractionBroadcast` from the server and apply the
/// interaction to the matching local entity.
///
/// This keeps door/switch state visually consistent when a remote player (or the
/// host player) toggles an interactive object. The interaction uses the same
/// `ExecuteInteractionEvent` path as local interactions, so all visual
/// updates, sound and board topology rebuilds work correctly.
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
        } else {
            warn!(
                "apply_remote_interaction: no entity found at board position {:?}",
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

/// Server: read each player's `PlayerGear` and copy live gear component state into
/// the corresponding `*Net` replicated components for propagation to clients.
///
/// Iterates all gear entities held by the player (left hand, right hand, inventory)
/// and matches components to update the appropriate net type.  Unknown gear slots are
/// left at their previous (or default) value.
fn sync_gear_to_net(
    mut q_players: Query<(
        &PlayerGear,
        &mut FlashlightNet,
        &mut ThermometerNet,
        &mut EMFMeterNet,
        &mut SpiritBoxNet,
        &mut SageBundleNet,
        &mut RepellentFlaskNet,
        &mut PlayerGearKindNet,
    )>,
    q_flashlight: Query<(&Flashlight, Option<&Battery>)>,
    q_thermometer: Query<(&Thermometer, Option<&EquipmentPosition>)>,
    q_emf: Query<(&EMFMeter, Option<&EquipmentPosition>, Option<&Toggleable>)>,
    q_spiritbox: Query<(&SpiritBox, Option<&Toggleable>)>,
    q_sage: Query<&SageBundleData>,
    q_repellent: Query<&RepellentFlask>,
    q_gear_kind: Query<&GearKind>,
) {
    for (
        gear,
        mut fl_net,
        mut th_net,
        mut emf_net,
        mut sb_net,
        mut sage_net,
        mut rf_net,
        mut kind_net,
    ) in q_players.iter_mut()
    {
        // Update the gear-kind roster from the server's authoritative slot entities.
        kind_net.left_hand = gear
            .left_hand
            .and_then(|e| q_gear_kind.get(e).ok().copied())
            .unwrap_or(GearKind::None);
        kind_net.right_hand = gear
            .right_hand
            .and_then(|e| q_gear_kind.get(e).ok().copied())
            .unwrap_or(GearKind::None);
        kind_net.inventory = [
            gear.inventory
                .first()
                .and_then(|&e| q_gear_kind.get(e).ok().copied())
                .unwrap_or(GearKind::None),
            gear.inventory
                .get(1)
                .and_then(|&e| q_gear_kind.get(e).ok().copied())
                .unwrap_or(GearKind::None),
        ];

        let all_entities: Vec<Entity> = gear
            .left_hand
            .iter()
            .chain(gear.right_hand.iter())
            .chain(gear.inventory.iter())
            .copied()
            .collect();

        for entity in all_entities {
            if let Ok((fl, battery)) = q_flashlight.get(entity) {
                fl_net.status = fl.status.clone();
                fl_net.battery = battery.map(|b| b.level).unwrap_or(1.0);
            }
            if let Ok((_, eq_pos)) = q_thermometer.get(entity) {
                th_net.is_deployed =
                    eq_pos.is_some_and(|p| matches!(p, EquipmentPosition::Deployed));
            }
            if let Ok((_, eq_pos, toggle)) = q_emf.get(entity) {
                emf_net.is_deployed =
                    eq_pos.is_some_and(|p| matches!(p, EquipmentPosition::Deployed));
                emf_net.is_on = toggle.is_some_and(|t| t.is_on);
            }
            if let Ok((sb, toggle)) = q_spiritbox.get(entity) {
                sb_net.charge = sb.charge;
                sb_net.is_on = toggle.is_some_and(|t| t.is_on);
            }
            if let Ok(sage) = q_sage.get(entity) {
                sage_net.consumed = sage.consumed;
                sage_net.is_active = sage.is_active;
                sage_net.remaining_secs = sage.burn_timer.remaining_secs();
            }
            if let Ok(rf) = q_repellent.get(entity) {
                rf_net.qty = rf.qty;
                rf_net.active = rf.active;
                rf_net.liquid_content = rf.liquid_content;
            }
        }
    }
}

/// Client: apply `PlayerStateNet` changes to the local `PlayerSprite` for remote players.
///
/// The local player's `PlayerSprite` is authoritative (not driven from net components),
/// so we exclude entities with `MainPlayer`.
fn apply_player_state_net(
    mut q: Query<
        (
            Entity,
            &PlayerStateNet,
            &mut PlayerSprite,
            Has<PlayerSpectating>,
        ),
        (Changed<PlayerStateNet>, Without<MainPlayer>),
    >,
    mut commands: Commands,
) {
    for (entity, net, mut sprite, already_spectating) in q.iter_mut() {
        sprite.health = net.health;
        sprite.sanity = net.sanity;
        // Mirror the authoritative spectating state so remote players visually
        // enter spectator mode when the server marks them as dead.
        if net.is_spectating && !already_spectating {
            commands.entity(entity).insert(PlayerSpectating);
        }
    }
}

/// Client: apply `ThermometerNet` changes to the gear entity's `EquipmentPosition`.
///
/// `is_deployed == true` inserts `EquipmentPosition::Deployed`; false removes it so the
/// gear renders as held. We set `Stowed` rather than trying to infer which hand it's in
/// since remote player hand-slot data is not available at this layer.
fn apply_thermometer_net(
    q_players: Query<
        (&PlayerGear, &ThermometerNet),
        (Changed<ThermometerNet>, Without<MainPlayer>),
    >,
    mut commands: Commands,
    q_thermometer: Query<Entity, With<Thermometer>>,
) {
    for (gear, th_net) in q_players.iter() {
        let all_entities = gear
            .left_hand
            .iter()
            .chain(gear.right_hand.iter())
            .chain(gear.inventory.iter())
            .copied();

        for entity in all_entities {
            if q_thermometer.get(entity).is_ok() {
                if th_net.is_deployed {
                    commands.entity(entity).insert(EquipmentPosition::Deployed);
                } else {
                    commands.entity(entity).insert(EquipmentPosition::Stowed);
                }
            }
        }
    }
}

/// Client: apply `EMFMeterNet` changes to the gear entity's `EquipmentPosition` and `Toggleable`.
fn apply_emf_net(
    q_players: Query<(&PlayerGear, &EMFMeterNet), (Changed<EMFMeterNet>, Without<MainPlayer>)>,
    mut commands: Commands,
    q_emf: Query<Entity, With<EMFMeter>>,
) {
    for (gear, emf_net) in q_players.iter() {
        let all_entities = gear
            .left_hand
            .iter()
            .chain(gear.right_hand.iter())
            .chain(gear.inventory.iter())
            .copied();

        for entity in all_entities {
            if q_emf.get(entity).is_ok() {
                if emf_net.is_deployed {
                    commands.entity(entity).insert(EquipmentPosition::Deployed);
                } else {
                    commands.entity(entity).insert(EquipmentPosition::Stowed);
                }
                commands.entity(entity).insert(Toggleable {
                    is_on: emf_net.is_on,
                });
            }
        }
    }
}

/// Client: apply `SpiritBoxNet` changes to the gear entity's `SpiritBox` and `Toggleable`.
fn apply_spiritbox_net(
    q_players: Query<(&PlayerGear, &SpiritBoxNet), (Changed<SpiritBoxNet>, Without<MainPlayer>)>,
    mut q_spiritbox: Query<(&mut SpiritBox, Option<&Toggleable>)>,
    mut commands: Commands,
) {
    for (gear, sb_net) in q_players.iter() {
        let all_entities = gear
            .left_hand
            .iter()
            .chain(gear.right_hand.iter())
            .chain(gear.inventory.iter())
            .copied();

        for entity in all_entities {
            if let Ok((mut sb, _toggle)) = q_spiritbox.get_mut(entity) {
                sb.charge = sb_net.charge;
                commands.entity(entity).insert(Toggleable {
                    is_on: sb_net.is_on,
                });
            }
        }
    }
}

/// Client: apply `FlashlightNet` changes to the gear entity's `Flashlight` + `Battery`.
///
/// Looks up the player entity's `PlayerGear` and iterates gear entities to find
/// the one carrying a `Flashlight`, then writes the replicated state back.
fn apply_flashlight_net(
    q_players: Query<(&PlayerGear, &FlashlightNet), (Changed<FlashlightNet>, Without<MainPlayer>)>,
    mut q_flashlight: Query<(&mut Flashlight, Option<&mut Battery>)>,
) {
    for (gear, fl_net) in q_players.iter() {
        let all_entities = gear
            .left_hand
            .iter()
            .chain(gear.right_hand.iter())
            .chain(gear.inventory.iter())
            .copied();

        for entity in all_entities {
            if let Ok((mut fl, battery)) = q_flashlight.get_mut(entity) {
                fl.status = fl_net.status.clone();
                if let Some(mut bat) = battery {
                    bat.level = fl_net.battery;
                }
            }
        }
    }
}

/// Client: apply `SageBundleNet` changes to the gear entity's `SageBundleData`.
fn apply_sage_net(
    q_players: Query<(&PlayerGear, &SageBundleNet), (Changed<SageBundleNet>, Without<MainPlayer>)>,
    mut q_sage: Query<&mut SageBundleData>,
) {
    for (gear, sage_net) in q_players.iter() {
        let all_entities = gear
            .left_hand
            .iter()
            .chain(gear.right_hand.iter())
            .chain(gear.inventory.iter())
            .copied();

        for entity in all_entities {
            if let Ok(mut sage) = q_sage.get_mut(entity) {
                sage.consumed = sage_net.consumed;
                sage.is_active = sage_net.is_active;
                // Remaining secs is informational; the local timer drives animations
                // but we seed it from the net value so it stays roughly in sync.
                let duration_secs = sage.burn_timer.duration().as_secs_f32();
                sage.burn_timer
                    .set_elapsed(std::time::Duration::from_secs_f32(
                        (duration_secs - sage_net.remaining_secs).max(0.0),
                    ));
            }
        }
    }
}

/// Client: apply `RepellentFlaskNet` changes to the gear entity's `RepellentFlask`.
fn apply_repellent_net(
    q_players: Query<
        (&PlayerGear, &RepellentFlaskNet),
        (Changed<RepellentFlaskNet>, Without<MainPlayer>),
    >,
    mut q_repellent: Query<&mut RepellentFlask>,
) {
    for (gear, rf_net) in q_players.iter() {
        let all_entities = gear
            .left_hand
            .iter()
            .chain(gear.right_hand.iter())
            .chain(gear.inventory.iter())
            .copied();

        for entity in all_entities {
            if let Ok(mut rf) = q_repellent.get_mut(entity) {
                rf.qty = rf_net.qty;
                rf.active = rf_net.active;
                rf.liquid_content = rf_net.liquid_content;
            }
        }
    }
}

/// Client: apply `PlayerGearKindNet` changes to rebuild remote players' local gear entities.
///
/// When the server changes a player's gear roster (due to truck loadout selection,
/// grab, or drop), this system despawns the old local gear entities and spawns
/// fresh ones of the correct `GearKind`. Inventory is treated as two fixed slots.
///
/// Runs only on join clients (`not(ServerState::Running)`) for non-local entities
/// (`Without<MainPlayer>`).
fn apply_gear_kind_net(
    mut q: Query<
        (&PlayerGearKindNet, &mut PlayerGear),
        (Changed<PlayerGearKindNet>, Without<MainPlayer>),
    >,
    q_kind: Query<&GearKind>,
    gear_registry: Res<GearSpawnerRegistry>,
    mut commands: Commands,
) {
    for (kind_net, mut p_gear) in q.iter_mut() {
        // --- Left hand ---
        let left_current = p_gear
            .left_hand
            .and_then(|e| q_kind.get(e).ok().copied())
            .unwrap_or(GearKind::None);
        if left_current != kind_net.left_hand {
            if let Some(e) = p_gear.left_hand.take() {
                commands.entity(e).despawn();
            }
            p_gear.left_hand = kind_net
                .left_hand
                .is_some()
                .then(|| gear_registry.spawn(&mut commands, kind_net.left_hand));
        }

        // --- Right hand ---
        let right_current = p_gear
            .right_hand
            .and_then(|e| q_kind.get(e).ok().copied())
            .unwrap_or(GearKind::None);
        if right_current != kind_net.right_hand {
            if let Some(e) = p_gear.right_hand.take() {
                commands.entity(e).despawn();
            }
            p_gear.right_hand = kind_net
                .right_hand
                .is_some()
                .then(|| gear_registry.spawn(&mut commands, kind_net.right_hand));
        }

        // --- Inventory (two fixed slots) ---
        // Process each slot in order, adjusting for removals as we go.
        for idx in 0..2usize {
            let desired = kind_net.inventory[idx];
            let current = p_gear
                .inventory
                .get(idx)
                .and_then(|&e| q_kind.get(e).ok().copied())
                .unwrap_or(GearKind::None);
            if current == desired {
                continue;
            }
            // Despawn and remove the old entity at this index if present.
            if let Some(&old_entity) = p_gear.inventory.get(idx) {
                commands.entity(old_entity).despawn();
                p_gear.inventory.remove(idx);
            }
            if desired.is_some() {
                let new_entity = gear_registry.spawn(&mut commands, desired);
                if idx <= p_gear.inventory.len() {
                    p_gear.inventory.insert(idx, new_entity);
                } else {
                    p_gear.inventory.push(new_entity);
                }
            }
        }

        // Drop any excess inventory entities beyond the two replicated slots.
        while p_gear.inventory.len() > 2 {
            if let Some(e) = p_gear.inventory.pop() {
                commands.entity(e).despawn();
            }
        }
    }
}
