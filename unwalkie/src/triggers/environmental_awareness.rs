use bevy::prelude::*;

use uncore::{
    resources::board_data::BoardData,
    states::{AppState, GameState},
};

use unwalkiecore::{WalkiePlay, events::WalkieEvent};

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

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        // Only trigger_darkness_level_system remains
        trigger_darkness_level_system.run_if(in_state(AppState::InGame)),
    );
}
