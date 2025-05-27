use bevy::prelude::*;
use bevy::time::Stopwatch;
use std::any::Any; // Added import

use uncore::{
    components::{
        board::position::Position, ghost_breach::GhostBreach, player_sprite::PlayerSprite,
    },
    resources::board_data::BoardData,
    states::{AppState, GameState},
};

use uncore::resources::roomdb::RoomDB;
use uncore::types::gear_kind::GearKind;
use ungear::components::playergear::PlayerGear;
use ungear::gear_usable::GearUsable;
use ungearitems::components::thermometer::Thermometer;
use unwalkiecore::{WalkiePlay, events::WalkieEvent};

/// System that monitors the player's exposure to darkness.
///
/// If the player is in-game, not in the truck, and the environment is very dark (exposure_lux < 0.1),
/// it accumulates time spent in darkness. If the player remains in darkness for more than 10 seconds,
/// a walkie-talkie warning event is triggered. The timer resets if the player leaves the dark or the game state changes.
fn trigger_darkness_level_system(
    time: Res<Time>,
    board_data: Res<BoardData>,
    roomdb: Res<RoomDB>,
    mut walkie_play: ResMut<WalkiePlay>,
    game_state: Res<State<GameState>>,
    app_state: Res<State<AppState>>,
    qp: Query<(&Position, &PlayerSprite)>,
    mut stopwatch: Local<Stopwatch>,
) {
    if app_state.get() != &AppState::InGame {
        stopwatch.reset();
        return;
    }
    if *game_state.get() != GameState::None {
        stopwatch.reset();
        return;
    }
    let Ok((player_pos, _)) = qp.get_single() else {
        return;
    };
    let player_bpos = player_pos.to_board_position();
    let player_room = roomdb.room_tiles.get(&player_bpos);

    if player_room.is_none() {
        // Player is not inside the location, no need to remind them.
        stopwatch.reset();
        return;
    }

    if board_data.exposure_lux < 0.7 {
        stopwatch.tick(time.delta()); // Changed from *seconds_dark += time.delta_secs();
        if stopwatch.elapsed_secs() > 2.0 {
            walkie_play.set(WalkieEvent::DarkRoomNoLightUsed, time.elapsed_secs_f64());
        }
    } else {
        stopwatch.reset(); // Changed from *seconds_dark = 0.0;
    }
}

/// Triggers a walkie-talkie event if the player is in the same room as a breach.
fn trigger_breach_showcase(
    time: Res<Time>,
    roomdb: Res<RoomDB>,
    mut walkie_play: ResMut<WalkiePlay>,
    game_state: Res<State<GameState>>,
    app_state: Res<State<AppState>>,
    qp: Query<(&Position, &PlayerSprite)>,
    q_breach: Query<&Position, With<GhostBreach>>,
    truck_button_query: Query<&uncore::components::truck_ui_button::TruckUIButton>, // Added
) {
    if app_state.get() != &AppState::InGame {
        return;
    }
    if *game_state.get() != GameState::None {
        return;
    }

    // Check if any evidence is confirmed
    for button_data in truck_button_query.iter() {
        if let uncore::types::truck_button::TruckButtonType::Evidence(_) = button_data.class {
            if button_data.status == uncore::types::truck_button::TruckButtonState::Pressed {
                return; // Don't fire if any evidence is confirmed
            }
        }
    }

    let Ok((player_pos, _)) = qp.get_single() else {
        return;
    };
    let player_bpos = player_pos.to_board_position();
    let player_room = roomdb.room_tiles.get(&player_bpos);
    for breach_pos in q_breach.iter() {
        let breach_bpos = breach_pos.to_board_position();
        let breach_room = roomdb.room_tiles.get(&breach_bpos);

        if player_room.is_some()
            && breach_room.is_some()
            && player_room == breach_room
            && breach_pos.distance(player_pos) < 3.0
        {
            walkie_play.set(WalkieEvent::BreachShowcase, time.elapsed_secs_f64());
            break;
        }
    }
}

/// Triggers a walkie-talkie event if the player is in the same room as the ghost.
fn trigger_ghost_showcase(
    time: Res<Time>,
    roomdb: Res<RoomDB>,
    mut walkie_play: ResMut<WalkiePlay>,
    game_state: Res<State<GameState>>,
    app_state: Res<State<AppState>>,
    qp: Query<(&Position, &PlayerSprite)>,
    q_ghost: Query<&Position, With<uncore::components::ghost_sprite::GhostSprite>>,
    truck_button_query: Query<&uncore::components::truck_ui_button::TruckUIButton>, // Added
) {
    if app_state.get() != &AppState::InGame {
        return;
    }
    if *game_state.get() != GameState::None {
        return;
    }

    // Check if any evidence is confirmed
    for button_data in truck_button_query.iter() {
        if let uncore::types::truck_button::TruckButtonType::Evidence(_) = button_data.class {
            if button_data.status == uncore::types::truck_button::TruckButtonState::Pressed {
                return; // Don't fire if any evidence is confirmed
            }
        }
    }

    let Ok((player_pos, _)) = qp.get_single() else {
        return;
    };
    let player_bpos = player_pos.to_board_position();
    let player_room = roomdb.room_tiles.get(&player_bpos);
    for ghost_pos in q_ghost.iter() {
        let ghost_bpos = ghost_pos.to_board_position();
        let ghost_room = roomdb.room_tiles.get(&ghost_bpos);
        if player_room.is_some() && ghost_room.is_some() && player_room == ghost_room {
            walkie_play.set(WalkieEvent::GhostShowcase, time.elapsed_secs_f64());
            break;
        }
    }
}

/// Triggers a walkie-talkie event if the player uses gear that requires darkness in a lit room.
fn trigger_room_lights_on_gear_needs_dark(
    time: Res<Time>,
    board_data: Res<BoardData>,
    roomdb: Res<RoomDB>,
    mut walkie_play: ResMut<WalkiePlay>,
    game_state: Res<State<GameState>>,
    app_state: Res<State<AppState>>,
    qp: Query<(
        &Position,
        &PlayerSprite,
        &ungear::components::playergear::PlayerGear,
    )>,
) {
    if app_state.get() != &AppState::InGame {
        return;
    }
    if *game_state.get() != GameState::None {
        return;
    }
    let Ok((player_pos, _player, player_gear)) = qp.get_single() else {
        return;
    };
    let player_bpos = player_pos.to_board_position();
    let player_room = roomdb.room_tiles.get(&player_bpos);

    if player_room.is_none() {
        return;
    }

    // Use GearUsable::needs_darkness for the right hand gear
    if player_gear.right_hand.needs_darkness()
        && player_gear.right_hand.is_enabled()
        && board_data.exposure_lux > 0.5
    {
        // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
        walkie_play.set(
            WalkieEvent::RoomLightsOnGearNeedsDark,
            time.elapsed_secs_f64(),
        );
    }
}

/// Triggers a walkie-talkie event if the player lingers with the thermometer in cold (1-10°C, not freezing) for a set duration.
fn trigger_thermometer_non_freezing_fixation(
    time: Res<Time>,
    mut walkie_play: ResMut<WalkiePlay>,
    game_state: Res<State<GameState>>,
    app_state: Res<State<AppState>>,
    mut stopwatch: Local<Stopwatch>,
    mut trigger_count: Local<u32>,
    qp: Query<(&PlayerGear, &PlayerSprite)>,
) {
    // Only allow 2 triggers per mission
    const MAX_TRIGGERS: u32 = 2;
    const REQUIRED_DURATION: f32 = 15.0;
    if *trigger_count >= MAX_TRIGGERS {
        return;
    }
    if app_state.get() != &AppState::InGame {
        stopwatch.reset();
        return;
    }
    if *game_state.get() != GameState::None {
        stopwatch.reset();
        return;
    }
    let Ok((player_gear, _)) = qp.get_single() else {
        stopwatch.reset();
        return;
    };
    // Check if right hand is a Thermometer and enabled
    if let GearKind::Thermometer = player_gear.right_hand.kind {
        if let Some(thermo) = player_gear
            .right_hand
            .data
            .as_ref()
            .and_then(|d| <dyn Any>::downcast_ref::<Thermometer>(d.as_ref()))
        {
            if thermo.enabled {
                let temp_c = uncore::kelvin_to_celsius(thermo.temp);
                if (1.0..=10.0).contains(&temp_c) {
                    stopwatch.tick(time.delta());
                    if stopwatch.elapsed_secs() > REQUIRED_DURATION {
                        // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
                        walkie_play.set(
                            WalkieEvent::ThermometerNonFreezingFixation,
                            time.elapsed_secs_f64(),
                        );
                        *trigger_count += 1;
                        stopwatch.reset();
                    }
                    return; // Return to avoid resetting stopwatch if conditions are met
                }
            }
        }
    }
    stopwatch.reset();
}

/// Registers the environmental awareness systems to the Bevy app.
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            trigger_darkness_level_system.run_if(in_state(AppState::InGame)),
            trigger_breach_showcase.run_if(in_state(AppState::InGame)),
            trigger_ghost_showcase.run_if(in_state(AppState::InGame)),
            trigger_room_lights_on_gear_needs_dark.run_if(in_state(AppState::InGame)),
            trigger_thermometer_non_freezing_fixation.run_if(in_state(AppState::InGame)),
        ),
    );
}
