use bevy::app::App;
use bevy::prelude::*;
use uncore::{
    components::{board::position::Position, player_sprite::PlayerSprite},
    resources::roomdb::RoomDB,
    states::{AppState, GameState},
};
use unwalkiecore::{WalkieEvent, WalkiePlay};

/// Triggers a warning if the player's sanity drops below 30% and they don't return to the truck within 20 seconds.
fn very_low_sanity_no_truck_return(
    mut walkie_play: ResMut<WalkiePlay>,
    qp: Query<(&PlayerSprite, &Position)>,
    roomdb: Res<RoomDB>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    if app_state.get() != &AppState::InGame || *game_state.get() != GameState::None {
        *timer = 0.0;
        return;
    }
    let Some((player, pos)) = qp.iter().next() else {
        return;
    };
    if player.sanity() >= 30.0 {
        *timer = 0.0;
        return;
    }
    let player_bpos = pos.to_board_position();
    if roomdb.room_tiles.get(&player_bpos).is_none() {
        // Player is not inside the location, reset timer
        *timer = 0.0;
        return;
    }
    *timer += time.delta().as_secs_f32();
    if *timer > 20.0 {
        walkie_play.set(
            WalkieEvent::VeryLowSanityNoTruckReturn,
            time.elapsed_secs_f64(),
        );
        *timer = 0.0;
    }
}

/// Triggers a warning if the player's health drops below 50% for 30 seconds while inside the location.
fn low_health_general_warning(
    mut walkie_play: ResMut<WalkiePlay>,
    qp: Query<(&PlayerSprite, &Position)>,
    roomdb: Res<RoomDB>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    if app_state.get() != &AppState::InGame || *game_state.get() != GameState::None {
        *timer = 0.0;
        return;
    }
    let Some((player, pos)) = qp.iter().next() else {
        return;
    };
    if player.health >= 50.0 {
        *timer = 0.0;
        return;
    }
    let player_bpos = pos.to_board_position();
    if roomdb.room_tiles.get(&player_bpos).is_none() {
        // Player is not inside the location, reset timer
        *timer = 0.0;
        return;
    }
    *timer += time.delta().as_secs_f32();
    if *timer > 30.0 {
        walkie_play.set(
            WalkieEvent::LowHealthGeneralWarning,
            time.elapsed_secs_f64(),
        );
        *timer = 0.0;
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, very_low_sanity_no_truck_return)
        .add_systems(Update, low_health_general_warning);
}
