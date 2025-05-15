use bevy::prelude::*;

use uncore::{
    resources::board_data::BoardData,
    states::{AppState, GameState},
};

use uncore::resources::roomdb::RoomDB;
use uncore::traits::gear_usable::GearUsable;
use unwalkiecore::{WalkiePlay, events::WalkieEvent};

/// System that monitors the player's exposure to darkness.
///
/// If the player is in-game, not in the truck, and the environment is very dark (exposure_lux < 0.1),
/// it accumulates time spent in darkness. If the player remains in darkness for more than 10 seconds,
/// a walkie-talkie warning event is triggered. The timer resets if the player leaves the dark or the game state changes.
fn trigger_darkness_level_system(
    time: Res<Time>,
    board_data: Res<BoardData>,
    mut walkie_play: ResMut<WalkiePlay>,
    game_state: Res<State<GameState>>,
    app_state: Res<State<AppState>>,
    mut seconds_dark: Local<f32>,
) {
    if app_state.get() != &AppState::InGame {
        *seconds_dark = 0.0;
        return;
    }
    if *game_state.get() != GameState::None {
        *seconds_dark = 0.0;
        return;
    }

    if board_data.exposure_lux < 0.1 {
        *seconds_dark += time.delta_secs();
        if *seconds_dark > 10.0 {
            walkie_play.set(WalkieEvent::DarkRoomNoLightUsed, time.elapsed_secs_f64());
        }
    } else {
        *seconds_dark = 0.0;
    }
}

/// Triggers a walkie-talkie event if the player is in the same room as a breach.
fn trigger_breach_showcase(
    time: Res<Time>,
    roomdb: Res<RoomDB>,
    mut walkie_play: ResMut<WalkiePlay>,
    game_state: Res<State<GameState>>,
    app_state: Res<State<AppState>>,
    qp: Query<(
        &uncore::components::board::position::Position,
        &uncore::components::player_sprite::PlayerSprite,
    )>,
    q_breach: Query<
        &uncore::components::board::position::Position,
        With<uncore::components::ghost_breach::GhostBreach>,
    >,
) {
    if app_state.get() != &AppState::InGame {
        return;
    }
    if *game_state.get() != GameState::None {
        return;
    }
    let Ok((player_pos, _)) = qp.get_single() else {
        return;
    };
    let player_bpos = player_pos.to_board_position();
    let player_room = roomdb.room_tiles.get(&player_bpos);
    for breach_pos in q_breach.iter() {
        let breach_bpos = breach_pos.to_board_position();
        let breach_room = roomdb.room_tiles.get(&breach_bpos);
        if player_room.is_some() && breach_room.is_some() && player_room == breach_room {
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
    qp: Query<(
        &uncore::components::board::position::Position,
        &uncore::components::player_sprite::PlayerSprite,
    )>,
    q_ghost: Query<
        &uncore::components::board::position::Position,
        With<uncore::components::ghost_sprite::GhostSprite>,
    >,
) {
    if app_state.get() != &AppState::InGame {
        return;
    }
    if *game_state.get() != GameState::None {
        return;
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
    mut walkie_play: ResMut<WalkiePlay>,
    game_state: Res<State<GameState>>,
    app_state: Res<State<AppState>>,
    qp: Query<(
        &uncore::components::board::position::Position,
        &uncore::components::player_sprite::PlayerSprite,
        &ungear::components::playergear::PlayerGear,
    )>,
) {
    if app_state.get() != &AppState::InGame {
        return;
    }
    if *game_state.get() != GameState::None {
        return;
    }
    let Ok((_player_pos, _player, player_gear)) = qp.get_single() else {
        return;
    };
    // Use GearUsable::needs_darkness for the right hand gear
    if player_gear.right_hand.needs_darkness() && board_data.exposure_lux > 0.5 {
        walkie_play.set(
            WalkieEvent::RoomLightsOnGearNeedsDark,
            time.elapsed_secs_f64(),
        );
    }
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
        ),
    );
}
