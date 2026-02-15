use crate::resources::NetworkConn;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use rand::prelude::*;
use unbehavior::behavior::Behavior;
use unboard_core::resources::board_topology::BoardEntityField;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use uninteraction_core::interaction::ExecuteInteractionEvent;
use unmapload_core::events::loadlevel::LoadLevelEvent;
use unmetrics_core::metrics::SendMetric;
use unnet_core::messages::{NetworkDataEvent, NetworkMessage, SendNetworkMessage};
use unnet_core::network_id::NetworkId;
use unnet_core::resources::{CurrentMapSeed, LobbyData, MissionEndRequested, RoomOwner};
use unplayer_core::components::{Hiding, MainPlayer, PlayerInput, PlayerSprite};
use unsettings_core::audio::AudioSettings;
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;
use untruck_core::types::repellent_tracker::RepellentCraftTracker;
use untypes_core::cli::{CliOptions, NetMode};
use untypes_core::difficulty::Difficulty;
use untypes_core::states::AppState;

use crate::metrics;

#[derive(SystemParam)]
#[allow(clippy::type_complexity)]
pub(crate) struct HostApplyInputParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub cli: Res<'w, CliOptions>,
    pub network_conn: Option<ResMut<'w, NetworkConn>>,
    pub ev_reader: MessageReader<'w, 's, NetworkDataEvent>,
    pub ev_send: MessageWriter<'w, SendNetworkMessage>,
    pub query_players: Query<
        'w,
        's,
        (
            Entity,
            &'static NetworkId,
            &'static mut PlayerInput,
            &'static mut PlayerGear,
            &'static mut Position,
            Has<InTruck>,
        ),
        (With<PlayerSprite>, Without<MainPlayer>, Without<Behavior>),
    >,
    pub query_van: Query<'w, 's, (&'static Position, &'static Behavior), Without<PlayerSprite>>,
    pub ev_interaction: MessageWriter<'w, ExecuteInteractionEvent>,
    pub board_field: Res<'w, BoardEntityField>,
    pub craft_tracker: ResMut<'w, RepellentCraftTracker>,
    pub gear_registry: Res<'w, GearSpawnerRegistry>,
    pub q_repellent:
        Query<'w, 's, &'static mut ungearitems_core::components::repellentflask::RepellentFlask>,
    pub q_gearkind: Query<'w, 's, &'static ungear_core::types::gear::kind::GearKind>,
    pub query_net_id: Query<'w, 's, &'static NetworkId>,
    pub asset_server: Res<'w, AssetServer>,
    pub audio_settings: Res<'w, Persistent<AudioSettings>>,
    pub ev_mission: MessageWriter<'w, unevents_core::events::mission::MissionEvent>,
    pub ev_load_level: MessageWriter<'w, LoadLevelEvent>,
    pub mission_end_requested: Res<'w, MissionEndRequested>,
    pub current_map_seed: Option<Res<'w, CurrentMapSeed>>,
    pub room_owner: Option<Res<'w, RoomOwner>>,
    pub lobby_data: Option<ResMut<'w, LobbyData>>,
    pub next_app_state: ResMut<'w, NextState<AppState>>,
    pub current_difficulty: ResMut<'w, CurrentDifficulty>,
}

pub(crate) fn host_apply_input_system(mut params: HostApplyInputParams) {
    let measure = metrics::HOST_APPLY_INPUT.time_measure();
    if !matches!(params.cli.net_mode, NetMode::Host { .. }) {
        measure.end_ms();
        return;
    }

    for ev in params.ev_reader.read() {
        match &ev.message {
            NetworkMessage::RequestLateJoin { player_id } => {
                let seed = params.current_map_seed.as_deref().map(|s| s.0).unwrap_or(0);
                let map_path = params
                    .lobby_data
                    .as_deref()
                    .and_then(|l| l.selected_map.clone())
                    .unwrap_or_default();
                let diff_id = params
                    .lobby_data
                    .as_deref()
                    .map(|l| l.selected_difficulty.clone())
                    .unwrap_or_else(|| "standard-challenge".to_string());

                debug!(
                    "Sending Late Join StartMission to {:?} (Map: {}, Diff: {}, Seed: {})",
                    player_id, map_path, diff_id, seed
                );

                if let Some(conn) = params.network_conn.as_deref_mut() {
                    conn.host_send_to(
                        *player_id,
                        NetworkMessage::StartMission {
                            map_seed: seed,
                            map_filepath: map_path,
                            difficulty_id: diff_id,
                        },
                    );

                    // Set needs_full_sync = true
                    if let NetworkConn::Host { clients, .. } = &mut *conn
                        && let Some(client) = clients
                            .iter_mut()
                            .find(|c| c.associated_id == Some(*player_id))
                    {
                        client.needs_full_sync = true;
                    }
                }
            }
            NetworkMessage::PlayerInput {
                player_id,
                o_position,
                movement,
                run,
                interact,
                use_right_hand,
                use_left_hand,
                target_right_hand,
                target_left_hand,
                target_position,
                aim_direction,
                sanity,
                mean_sound,
            } => {
                let mut found_player = false;
                for (_entity, id, mut input, _, mut pos, _) in params.query_players.iter_mut() {
                    if id == player_id {
                        found_player = true;
                        if let Some(position) = o_position {
                            pos.x = position[0];
                            pos.y = position[1];
                            pos.z = position[2];
                        }
                        input.movement = Vec2::new(movement[0], movement[1]);
                        input.run = *run;
                        input.interact = *interact;
                        input.use_right_hand = *use_right_hand;
                        input.use_left_hand = *use_left_hand;
                        input.target_right_hand = target_right_hand.clone();
                        input.target_left_hand = target_left_hand.clone();
                        input.target_position = target_position.map(|v| Vec2::new(v[0], v[1]));
                        input.aim_direction = Vec2::new(aim_direction[0], aim_direction[1]);
                        input.sanity = *sanity;
                        input.mean_sound = *mean_sound;
                    }
                }
                if !found_player {
                    warn!(
                        "[WAYPOINT-TRACE] host_apply_input: NO ENTITY found for player_id={:?}",
                        player_id
                    );
                }
            }
            NetworkMessage::InteractionRequest {
                player_id: _,
                position,
                interaction_type,
            } => {
                let rel_x = position[0] as i64;
                let rel_y = position[1] as i64;
                let rel_z = position[2] as i64;

                if rel_x >= 0
                    && rel_y >= 0
                    && rel_z >= 0
                    && rel_x < params.board_field.0.shape()[0] as i64
                    && rel_y < params.board_field.0.shape()[1] as i64
                    && rel_z < params.board_field.0.shape()[2] as i64
                {
                    let entities =
                        &params.board_field.0[[rel_x as usize, rel_y as usize, rel_z as usize]];
                    for &entity in entities {
                        params.ev_interaction.write(ExecuteInteractionEvent {
                            entity,
                            ietype: interaction_type.clone(),
                            force_tuid: None,
                        });
                    }
                }
            }
            NetworkMessage::RequestTruckEntry { player_id } => {
                debug!(
                    "Network: Received RequestTruckEntry from client {:?}",
                    player_id
                );
                for (entity, id, _, _, p_pos, _) in params.query_players.iter() {
                    if id == player_id {
                        let mut near_van = false;
                        for (v_pos, v_beh) in params.query_van.iter() {
                            if v_beh.is_van_entry() && p_pos.distance(v_pos) < 2.0 {
                                near_van = true;
                                break;
                            }
                        }

                        if near_van {
                            info!(
                                "Network: RequestTruckEntry validated for player {:?}. Adding InTruck.",
                                player_id
                            );
                            params
                                .commands
                                .entity(entity)
                                .insert(InTruck)
                                .insert(Hiding { hiding_spot: None });
                        } else {
                            warn!(
                                "Network: Rejected RequestTruckEntry for player {:?} - too far from van",
                                player_id
                            );
                        }
                        break;
                    }
                }
            }
            NetworkMessage::RequestTruckExit { player_id } => {
                debug!(
                    "Network: Received RequestTruckExit from client {:?}",
                    player_id
                );
                for (entity, id, _, _, _, _) in params.query_players.iter() {
                    if id == player_id {
                        params
                            .commands
                            .entity(entity)
                            .remove::<InTruck>()
                            .remove::<Hiding>();
                        break;
                    }
                }
            }
            NetworkMessage::RequestHide { player_id } => {
                debug!("Network: Received RequestHide from client {:?}", player_id);
                for (entity, id, _, _, _, _) in params.query_players.iter() {
                    if id == player_id {
                        params
                            .commands
                            .entity(entity)
                            .insert(Hiding { hiding_spot: None });
                        break;
                    }
                }
            }
            NetworkMessage::RequestUnhide { player_id } => {
                debug!(
                    "Network: Received RequestUnhide from client {:?}",
                    player_id
                );
                for (entity, id, _, _, _, _) in params.query_players.iter() {
                    if id == player_id {
                        params.commands.entity(entity).remove::<Hiding>();
                        break;
                    }
                }
            }
            NetworkMessage::RequestEndMission => {
                debug!("Network: Received RequestEndMission from client. Validating.");
                if params.mission_end_requested.0 {
                    info!("Network: RequestEndMission validated. Ending mission.");
                    params
                        .ev_mission
                        .write(unevents_core::events::mission::MissionEvent::End);
                } else {
                    warn!("Network: RequestEndMission received but conditions not met.");
                }
            }
            NetworkMessage::CraftRepellent {
                player_id,
                ghost_type,
            } => {
                debug!(
                    "Network: Received CraftRepellent from client {:?} for {:?}",
                    player_id, ghost_type
                );
                // 1. Validate player is in truck
                let mut p_data = None;
                for (entity, id, _, _, _, is_in_truck) in params.query_players.iter() {
                    if id == player_id {
                        p_data = Some((entity, is_in_truck));
                        break;
                    }
                }

                if let Some((_entity, is_in_truck)) = p_data {
                    if is_in_truck {
                        // 2. Validate craft limits
                        if params.craft_tracker.can_craft() {
                            // 3. Perform craft
                            // Find the gear for this player
                            for (_e, id, _, mut gear, _pos, _) in params.query_players.iter_mut() {
                                if id == player_id {
                                    let consumed_new_bottle =
                                        untruck_plugin::craft_repellent::craft_repellent(
                                            &mut params.commands,
                                            &params.gear_registry,
                                            &mut gear,
                                            *ghost_type,
                                            &mut params.q_repellent,
                                            &params.q_gearkind,
                                        );

                                    // Ensure any newly spawned gear has a NetworkId
                                    let mut ensure_net_id = |entity: Entity| {
                                        if params.query_net_id.get(entity).is_err() {
                                            let mut rng = rand::rng();
                                            let net_id =
                                                NetworkId(rng.random_range(1000..u64::MAX));
                                            params.commands.entity(entity).insert(net_id);
                                            debug!(
                                                "Network: Assigned {:?} to local entity {:?}",
                                                net_id, entity
                                            );
                                        }
                                    };
                                    if let Some(e) = gear.left_hand {
                                        ensure_net_id(e);
                                    }
                                    if let Some(e) = gear.right_hand {
                                        ensure_net_id(e);
                                    }

                                    if consumed_new_bottle {
                                        params.craft_tracker.craft();
                                        info!(
                                            "Network: Crafted repellent for remote player {:?}",
                                            player_id
                                        );

                                        // Play sound at player position
                                        if !params.cli.is_headless() {
                                            params
                                                .commands
                                                .spawn(AudioPlayer::new(
                                                    params
                                                        .asset_server
                                                        .load("sounds/effects-dingdingding.ogg"),
                                                ))
                                                .insert(PlaybackSettings {
                                                    mode: bevy::audio::PlaybackMode::Despawn,
                                                    volume: bevy::audio::Volume::Linear(
                                                        1.0 * params
                                                            .audio_settings
                                                            .volume_master
                                                            .as_f32()
                                                            * params
                                                                .audio_settings
                                                                .volume_effects
                                                                .as_f32(),
                                                    ),
                                                    ..Default::default()
                                                });
                                        }
                                    }

                                    // Client should close UI themselves upon receiving the update,
                                    // or we could send a command.
                                    // For now, removing InTruck will force them out if they sync state.
                                    // But they stay in Truck state locally.
                                    break;
                                }
                            }
                        } else {
                            warn!(
                                "Network: CraftRepellent rejected for {:?} - limit reached",
                                player_id
                            );
                        }
                    } else {
                        warn!(
                            "Network: CraftRepellent rejected for {:?} - not in truck",
                            player_id
                        );
                    }
                }
            }
            NetworkMessage::RequestTruckInventoryChange { player_id, change } => {
                for (_entity, id, _input, mut p_gear, _pos, _) in params.query_players.iter_mut() {
                    if id == player_id {
                        match change {
                            unnet_core::messages::TruckInventoryChange::RemoveLeftHand => {
                                if let Some(e) = p_gear.left_hand.take() {
                                    params.commands.entity(e).despawn();
                                }
                            }
                            unnet_core::messages::TruckInventoryChange::RemoveRightHand => {
                                if let Some(e) = p_gear.right_hand.take() {
                                    params.commands.entity(e).despawn();
                                }
                            }
                            unnet_core::messages::TruckInventoryChange::RemoveInventoryIndex(
                                idx,
                            ) => {
                                if *idx < p_gear.inventory.len() {
                                    let e = p_gear.inventory.remove(*idx);
                                    params.commands.entity(e).despawn();
                                }
                            }
                            unnet_core::messages::TruckInventoryChange::AddItem(kind) => {
                                if *kind != ungear_core::types::gear::kind::GearKind::None {
                                    let entity =
                                        params.gear_registry.spawn(&mut params.commands, *kind);
                                    let mut rng = rand::rng();
                                    let net_id = NetworkId(rng.random_range(1000..u64::MAX));
                                    params.commands.entity(entity).insert(net_id);

                                    if p_gear.left_hand.is_none() {
                                        p_gear.left_hand = Some(entity);
                                    } else if p_gear.right_hand.is_none() {
                                        p_gear.right_hand = Some(entity);
                                    } else if p_gear.inventory.len() < 2 {
                                        p_gear.inventory.push(entity);
                                    } else {
                                        params.commands.entity(entity).despawn();
                                    }
                                }
                            }
                        }
                        // Issue 2 Fix: Force full sync when inventory changes relative to the truck
                        // This ensures clients process the despawned entities correctly
                        if let Some(conn) = &mut params.network_conn
                            && let NetworkConn::Host { clients, .. } = &mut **conn
                        {
                            for client in clients.iter_mut() {
                                if client.associated_id == Some(*player_id) {
                                    client.needs_full_sync = true;
                                    break;
                                }
                            }
                        }
                        break;
                    }
                }
            }
            NetworkMessage::GrabRequest(msg) => {
                for (_, id, mut input, _, _pos, _) in params.query_players.iter_mut() {
                    if id == &msg.player_id {
                        input.grab = true;
                    }
                }
            }
            NetworkMessage::DropRequest { player_id } => {
                for (_, id, mut input, _, _pos, _) in params.query_players.iter_mut() {
                    if id == player_id {
                        input.drop = true;
                    }
                }
            }
            NetworkMessage::CycleInventoryRequest { player_id } => {
                for (_, id, mut input, _, _pos, _) in params.query_players.iter_mut() {
                    if id == player_id {
                        input.inventory_cycle = true;
                    }
                }
            }
            NetworkMessage::SwapHandsRequest { player_id } => {
                for (_, id, mut input, _, _pos, _) in params.query_players.iter_mut() {
                    if id == player_id {
                        input.inventory_swap = true;
                    }
                }
            }
            NetworkMessage::PlayerLeft { player_id } => {
                info!("Network: Received PlayerLeft from client {:?}", player_id);
                for (entity, id, _, _, _, _) in params.query_players.iter() {
                    if id == player_id {
                        info!("Network: Despawning player entity for {:?}", id);
                        params.commands.entity(entity).despawn();
                        break;
                    }
                }
            }
            NetworkMessage::RequestFullSync { player_id } => {
                info!(
                    "Network: Received RequestFullSync from client {:?}",
                    player_id
                );
                if let Some(conn) = &mut params.network_conn
                    && let NetworkConn::Host { clients, .. } = &mut **conn
                {
                    // Find the client that sent this request
                    for client in clients.iter_mut() {
                        if client.associated_id == Some(*player_id) {
                            client.needs_full_sync = true;
                            break;
                        }
                    }
                }
            }
            NetworkMessage::RequestSelectMap {
                player_id,
                map_filepath,
            } => {
                if params.room_owner.as_deref() == Some(&RoomOwner(*player_id))
                    && let Some(lobby_data) = params.lobby_data.as_mut()
                {
                    lobby_data.selected_map = Some(map_filepath.clone());
                }
            }
            NetworkMessage::RequestSelectDifficulty {
                player_id,
                difficulty_id,
            } => {
                if params.room_owner.as_deref() == Some(&RoomOwner(*player_id))
                    && let Some(lobby_data) = params.lobby_data.as_mut()
                {
                    lobby_data.selected_difficulty = difficulty_id.clone();
                }
            }
            NetworkMessage::RequestStartMission { player_id } => {
                if params.room_owner.as_deref() == Some(&RoomOwner(*player_id)) {
                    let mut rng = rand::rng();
                    let new_seed: u64 = rng.random();
                    params.commands.insert_resource(CurrentMapSeed(new_seed));

                    let map = params
                        .lobby_data
                        .as_ref()
                        .and_then(|ld| ld.selected_map.clone())
                        .unwrap_or_default();

                    if map.is_empty() {
                        warn!("RequestStartMission: No map selected, ignoring.");
                        return;
                    }

                    let diff = params
                        .lobby_data
                        .as_ref()
                        .map(|ld| ld.selected_difficulty.clone())
                        .unwrap_or_else(|| "standard-challenge".to_string());

                    params
                        .ev_send
                        .write(SendNetworkMessage(NetworkMessage::StartMission {
                            map_filepath: map.clone(),
                            map_seed: new_seed,
                            difficulty_id: diff.clone(),
                        }));

                    // Note: The transition to InGame state should happen after the map is loaded.
                    // For dedicated server, we trigger the load and transition to Loading.
                    if params.cli.is_headless() {
                        use std::str::FromStr;
                        if let Ok(d) = Difficulty::from_str(&diff) {
                            *params.current_difficulty = CurrentDifficulty::new(d);
                        }
                        params
                            .ev_load_level
                            .write(LoadLevelEvent { map_filepath: map });
                        params.next_app_state.set(AppState::Loading);
                    }
                }
            }
            _ => {}
        }
    }
    measure.end_ms();
}
