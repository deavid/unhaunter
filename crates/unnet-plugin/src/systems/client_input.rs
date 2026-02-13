use crate::resources::NetworkConn;
use bevy::prelude::*;
use unassets_core::resources::maps::Maps;
use ungear_core::components::playergear::PlayerGear;
use uninteraction_core::interaction::Toggleable;
use unmapload_core::events::loadlevel::LoadLevelEvent;
use unmetrics_core::metrics::SendMetric;
use unnet_core::messages::{NetworkDataEvent, NetworkMessage};
use unnet_core::network_id::NetworkId;
use unnet_core::resources::LocalPlayer;
use unplayer_core::components::{MainPlayer, PlayerInput, PlayerSprite};
use unspatial_core::position::Position;
use untypes_core::cli::{CliOptions, NetMode};

use super::utils::extract_gear_details;
use crate::metrics;

pub(crate) fn client_sync_intended_gear_state(
    mut q_player: Query<(&PlayerGear, &mut PlayerInput), With<MainPlayer>>,
    q_gear: Query<(
        &Toggleable,
        Option<&ungearitems_core::components::flashlight::Flashlight>,
        Option<&ungearitems_core::components::sage::SageBundleData>,
        Option<&ungearitems_core::components::repellentflask::RepellentFlask>,
        Option<&ungearitems_core::components::thermometer::Thermometer>,
        Option<&ungearitems_core::components::emfmeter::EMFMeter>,
        Option<&ungearitems_core::components::spiritbox::SpiritBox>,
    )>,
    cli: Res<CliOptions>,
) {
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        return;
    }
    // Send current gear state every frame (not just on click).
    // The host copies this state directly onto the remote player's gear,
    // mirroring the client-authoritative pattern used for player position.
    for (gear, mut input) in q_player.iter_mut() {
        if let Some(entity) = gear.right_hand
            && let Ok((toggle, f, s, r, t, e, sb)) = q_gear.get(entity)
        {
            let details = extract_gear_details(f, s, r, t, e, sb);
            input.target_right_hand = Some((toggle.is_on, details));
        }
        if let Some(entity) = gear.left_hand
            && let Ok((toggle, f, s, r, t, e, sb)) = q_gear.get(entity)
        {
            let details = extract_gear_details(f, s, r, t, e, sb);
            input.target_left_hand = Some((toggle.is_on, details));
        }
    }
}

pub(crate) fn client_send_input_system(
    mut conn: ResMut<NetworkConn>,
    cli: Res<CliOptions>,
    local_id: Res<LocalPlayer>,
    query_player: Query<(&PlayerInput, &Position, &PlayerSprite), With<MainPlayer>>,
    mut ev_net_data: MessageReader<NetworkDataEvent>,
    mut pending_map: ResMut<crate::resources::PendingMapLoad>,
    mut local_tick: Local<u64>,
) {
    let measure = metrics::CLIENT_SEND_INPUT.time_measure();
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        measure.end_ms();
        return;
    }
    if !conn.is_active() {
        measure.end_ms();
        return;
    }

    let Some(player_id) = local_id.0 else {
        measure.end_ms();
        return;
    };

    *local_tick += 1;

    // After the client finishes loading the map and enters InGame, request a
    // full state sync so doors, tiles, etc. reflect the host's current state.
    if pending_map.needs_full_sync_request {
        info!("Network: Sending RequestFullSync to host");
        conn.client_send(NetworkMessage::RequestFullSync { player_id });
        pending_map.needs_full_sync_request = false;
        *local_tick = 0;
    }

    for (input, pos, sprite) in query_player.iter() {
        let o_position = if *local_tick < 10 {
            None
        } else {
            Some([pos.x, pos.y, pos.z])
        };
        conn.client_send(NetworkMessage::PlayerInput {
            player_id,
            o_position,
            movement: [input.movement.x, input.movement.y],
            run: input.run,
            interact: input.interact,
            use_right_hand: input.use_right_hand,
            use_left_hand: input.use_left_hand,
            target_right_hand: input.target_right_hand.clone(),
            target_left_hand: input.target_left_hand.clone(),
            target_position: input.target_position.map(|v| [v.x, v.y]),
            aim_direction: [input.aim_direction.x, input.aim_direction.y],
            sanity: sprite.sanity,
            mean_sound: sprite.mean_sound,
        });
    }

    // Pass through CraftRepellent, Truck entry/exit, and Interaction requests
    for ev in ev_net_data.read() {
        match ev.message {
            // Messages we know we must process:
            NetworkMessage::CraftRepellent { .. }
            | NetworkMessage::RequestTruckEntry { .. }
            | NetworkMessage::RequestTruckExit { .. }
            | NetworkMessage::RequestHide { .. }
            | NetworkMessage::RequestUnhide { .. }
            | NetworkMessage::InteractionRequest { .. }
            | NetworkMessage::RequestSelectMap { .. }
            | NetworkMessage::RequestSelectDifficulty { .. }
            | NetworkMessage::RequestStartMission { .. } => {
                conn.client_send(ev.message.clone());
            }
            // Messages that we know we must NOT process:
            NetworkMessage::Snapshot(_) => {}
            // Other messages, we report them just in case:
            _ => {
                trace!("Ignoring message - not sending to the host: {ev:?}");
            }
        }
    }
    measure.end_ms();
}

pub(crate) fn client_process_pending_map(
    mut pending_map: ResMut<crate::resources::PendingMapLoad>,
    maps: Res<Maps>,
    mut ev_load_level: MessageWriter<LoadLevelEvent>,
) {
    let measure = metrics::CLIENT_PROCESS_PENDING_MAP.time_measure();
    // Clone path to avoid holding borrow on pending_map
    let Some(path) = pending_map.map_filepath.clone() else {
        measure.end_ms();
        return;
    };

    // Check if that path exists in maps.maps
    if maps.maps.iter().any(|m| m.path == path) {
        info!("Network: Map assets ready. Loading map: {}", path);
        ev_load_level.write(LoadLevelEvent {
            map_filepath: path.clone(),
        });
        pending_map.map_filepath = None;
        pending_map.needs_full_sync_request = true;
    } else {
        trace!("Network: Waiting for map asset to be ready: {}", path);
    }
    measure.end_ms();
}

pub(crate) fn client_request_grab_system(
    mut conn: ResMut<NetworkConn>,
    cli: Res<CliOptions>,
    local_id: Res<LocalPlayer>,
    query_player: Query<&PlayerInput, With<MainPlayer>>,
) {
    let measure = metrics::CLIENT_REQUEST_GRAB.time_measure();
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        measure.end_ms();
        return;
    }
    if !conn.is_active() {
        measure.end_ms();
        return;
    }

    let Some(player_id) = local_id.0 else {
        measure.end_ms();
        return;
    };

    for input in query_player.iter() {
        if input.grab {
            conn.client_send(NetworkMessage::GrabRequest(
                unnet_core::messages::GrabRequestMsg {
                    player_id,
                    target_id: NetworkId::default(),
                },
            ));
        }
        if input.drop {
            conn.client_send(NetworkMessage::DropRequest { player_id });
        }
        if input.inventory_cycle {
            conn.client_send(NetworkMessage::CycleInventoryRequest { player_id });
        }
        if input.inventory_swap {
            conn.client_send(NetworkMessage::SwapHandsRequest { player_id });
        }
    }
    measure.end_ms();
}
