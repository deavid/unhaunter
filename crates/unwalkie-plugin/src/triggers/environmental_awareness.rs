use bevy::prelude::*;
use bevy::time::Stopwatch;

use uncommon_states_core::UIContextState;
use unghost_core::components::logic::ghost_breach::GhostBreach;
use unlight_core::resources::light_grid::LightGrid;
use unplayer_core::components::{MainPlayer, PlayerSprite};
use unspatial_core::position::Position;

use unboard_core::resources::roomdb::RoomTopology;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::types::gear::kind::GearKind;
use ungearitems_core::components::thermometer::Thermometer;
use uninteraction_core::interaction::Toggleable;
use unwalkie_core::events::walkie_types::WalkieEvent;
use unwalkie_core::messages::ProposeWalkieEvent;
use unwalkie_core::resources::WalkiePlay;

/// System that monitors the player's exposure to darkness.
///
/// If the player is in-game, not in the truck, and the environment is very dark (exposure.lux < 0.1),
/// it accumulates time spent in darkness. If the player remains in darkness for more than 10 seconds,
/// a walkie-talkie warning event is triggered. The timer resets if the player leaves the dark or the game state changes.
fn trigger_darkness_level_system(
    time: Res<Time>,
    light_grid: If<Res<LightGrid>>,
    room_topology: Res<RoomTopology>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    app_state: Res<State<UIContextState>>,
    qp: Query<(&Position, &PlayerSprite), With<MainPlayer>>,
    mut stopwatch: Local<Stopwatch>,
) {
    if app_state.get() != &UIContextState::InGame {
        stopwatch.reset();
        return;
    }
    let mut any_in_dark = false;
    for (player_pos, _) in qp.iter() {
        let player_bpos = player_pos.to_board_position();
        let player_room = room_topology.room_tiles.get(&player_bpos);

        if player_room.is_some() && light_grid.exposure.lux < 0.4 {
            any_in_dark = true;
            break;
        }
    }

    if any_in_dark {
        stopwatch.tick(time.delta()); // Changed from *seconds_dark += time.delta_secs();
        if stopwatch.elapsed_secs() > 2.0 {
            crate::triggers::net::walkie_set_or_propose(
                WalkieEvent::DarkRoomNoLightUsed,
                time.elapsed_secs_f64(),
                &mut walkie_play,
                &mut ev_propose,
            );
        }
    } else {
        stopwatch.reset(); // Changed from *seconds_dark = 0.0;
    }
}

/// Triggers a walkie-talkie event if the player is in the same room as a breach.
fn trigger_breach_showcase(
    time: Res<Time>,
    room_topology: Res<RoomTopology>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    app_state: Res<State<UIContextState>>,
    qp: Query<(&Position, &PlayerSprite), With<MainPlayer>>,
    q_breach: Query<&Position, With<GhostBreach>>,
    truck_button_query: Query<&untruck_core::components::truck_ui_button::TruckUIButton>, // Added
) {
    if app_state.get() != &UIContextState::InGame {
        return;
    }

    // Check if any evidence is confirmed
    for button_data in truck_button_query.iter() {
        if let untruck_core::types::truck_button::TruckButtonType::Evidence(_) = button_data.class
            && button_data.status == untruck_core::types::truck_button::TruckButtonState::Pressed
        {
            return; // Don't fire if any evidence is confirmed
        }
    }

    for (player_pos, _) in qp.iter() {
        let player_bpos = player_pos.to_board_position();
        let player_room = room_topology.room_tiles.get(&player_bpos);
        for breach_pos in q_breach.iter() {
            let breach_bpos = breach_pos.to_board_position();
            let breach_room = room_topology.room_tiles.get(&breach_bpos);

            if player_room.is_some()
                && breach_room.is_some()
                && player_room == breach_room
                && breach_pos.distance(player_pos) < 3.0
                && crate::triggers::net::walkie_set_or_propose(
                    WalkieEvent::BreachShowcase,
                    time.elapsed_secs_f64(),
                    &mut walkie_play,
                    &mut ev_propose,
                )
            {
                return;
            }
        }
    }
}

/// Triggers a walkie-talkie event if the player is in the same room as the ghost.
fn trigger_ghost_showcase(
    time: Res<Time>,
    room_topology: Res<RoomTopology>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    app_state: Res<State<UIContextState>>,
    qp: Query<(&Position, &PlayerSprite), With<MainPlayer>>,
    q_ghost: Query<&Position, With<unghost_core::components::logic::ghost_sprite::GhostSprite>>,
    truck_button_query: Query<&untruck_core::components::truck_ui_button::TruckUIButton>, // Added
) {
    if app_state.get() != &UIContextState::InGame {
        return;
    }

    // Check if any evidence is confirmed
    for button_data in truck_button_query.iter() {
        if let untruck_core::types::truck_button::TruckButtonType::Evidence(_) = button_data.class
            && button_data.status == untruck_core::types::truck_button::TruckButtonState::Pressed
        {
            return; // Don't fire if any evidence is confirmed
        }
    }

    for (player_pos, _) in qp.iter() {
        let player_bpos = player_pos.to_board_position();
        let player_room = room_topology.room_tiles.get(&player_bpos);
        for ghost_pos in q_ghost.iter() {
            let ghost_bpos = ghost_pos.to_board_position();
            let ghost_room = room_topology.room_tiles.get(&ghost_bpos);
            if player_room.is_some()
                && ghost_room.is_some()
                && player_room == ghost_room
                && crate::triggers::net::walkie_set_or_propose(
                    WalkieEvent::GhostShowcase,
                    time.elapsed_secs_f64(),
                    &mut walkie_play,
                    &mut ev_propose,
                )
            {
                return;
            }
        }
    }
}

/// Triggers a walkie-talkie event if the player uses gear that requires darkness in a lit room.
fn trigger_room_lights_on_gear_needs_dark(
    time: Res<Time>,
    light_grid: If<Res<LightGrid>>,
    room_topology: Res<RoomTopology>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    app_state: Res<State<UIContextState>>,
    qp: Query<(&Position, &PlayerSprite, &PlayerGear), With<MainPlayer>>,
    q_gear: Query<(&Toggleable, &GearKind)>,
) {
    if app_state.get() != &UIContextState::InGame {
        return;
    }
    for (player_pos, _player, player_gear) in qp.iter() {
        let player_bpos = player_pos.to_board_position();
        let player_room = room_topology.room_tiles.get(&player_bpos);

        if player_room.is_none() {
            continue;
        }

        // Use GearUsable::needs_darkness for the right hand gear
        if let Some(hand_entity) = player_gear.right_hand
            && let Ok((toggleable, kind)) = q_gear.get(hand_entity)
        {
            let needs_darkness = matches!(kind, GearKind::UVTorch | GearKind::Flashlight);
            if needs_darkness
                && toggleable.is_on
                && light_grid.light_field[player_bpos.ndidx()].lux > 0.5
            {
                // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
                if crate::triggers::net::walkie_set_or_propose(
                    WalkieEvent::RoomLightsOnGearNeedsDark,
                    time.elapsed_secs_f64(),
                    &mut walkie_play,
                    &mut ev_propose,
                ) {
                    return;
                }
            }
        }
    }
}

/// Triggers a walkie-talkie event if the player lingers with the thermometer in cold (1-10°C, not freezing) for a set duration.
fn trigger_thermometer_non_freezing_fixation(
    time: Res<Time>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    app_state: Res<State<UIContextState>>,
    mut stopwatch: Local<Stopwatch>,
    mut trigger_count: Local<u32>,
    qp: Query<(&PlayerGear, &PlayerSprite)>,
    q_thermometer: Query<(&Thermometer, &Toggleable)>,
) {
    // Only allow 2 triggers per mission
    const MAX_TRIGGERS: u32 = 2;
    const REQUIRED_DURATION: f32 = 15.0;
    if *trigger_count >= MAX_TRIGGERS {
        return;
    }
    if app_state.get() != &UIContextState::InGame {
        stopwatch.reset();
        return;
    }
    let mut any_player_fixing_on_cold = false;
    for (player_gear, _) in qp.iter() {
        // Check if right hand is a Thermometer and enabled
        if let Some(hand_entity) = player_gear.right_hand
            && let Ok((thermo, toggleable)) = q_thermometer.get(hand_entity)
        {
            let temp_c = uncommon_app_core::utils::temperature::kelvin_to_celsius(thermo.temp);
            if toggleable.is_on && (1.0..=10.0).contains(&temp_c) {
                any_player_fixing_on_cold = true;
                break;
            }
        }
    }

    if any_player_fixing_on_cold {
        stopwatch.tick(time.delta());
        if stopwatch.elapsed_secs() > REQUIRED_DURATION {
            // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
            crate::triggers::net::walkie_set_or_propose(
                WalkieEvent::ThermometerNonFreezingFixation,
                time.elapsed_secs_f64(),
                &mut walkie_play,
                &mut ev_propose,
            );
            *trigger_count += 1;
            stopwatch.reset();
        }
    } else {
        stopwatch.reset();
    }
}

/// Registers the environmental awareness systems to the Bevy app.
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            trigger_darkness_level_system.run_if(in_state(UIContextState::InGame)),
            trigger_breach_showcase.run_if(in_state(UIContextState::InGame)),
            trigger_ghost_showcase.run_if(in_state(UIContextState::InGame)),
            trigger_room_lights_on_gear_needs_dark.run_if(in_state(UIContextState::InGame)),
            trigger_thermometer_non_freezing_fixation.run_if(in_state(UIContextState::InGame)),
        ),
    );
}
