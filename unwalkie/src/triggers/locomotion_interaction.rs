use bevy::prelude::*;

use uncore::components::board::direction::Direction;
use uncore::components::{board::position::Position, player_sprite::PlayerSprite};
use uncore::resources::roomdb::RoomDB;
use uncore::states::{AppState, GameState};
use unwalkiecore::{WalkieEvent, WalkiePlay};

const PLAYER_STUCK_AT_START_SECONDS: f32 = 30.0;
const PLAYER_STUCK_MAX_DISTANCE: f32 = 1.0;
const ERRATIC_MOVEMENT_EARLY_SECONDS: f32 = 2.0;
const PLAYER_ERRATIC_MAX_DISTANCE: f32 = 6.0;

fn check_player_stuck_at_start(
    time: Res<Time>,
    game_state: Res<State<GameState>>,
    app_state: Res<State<AppState>>,
    roomdb: Res<RoomDB>,
    player_query: Query<(&Position, &PlayerSprite)>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut stuck_time: Local<f32>,
) {
    if app_state.get() != &AppState::InGame {
        *stuck_time = 0.0;
        return;
    }
    if *game_state.get() != GameState::None {
        *stuck_time = 0.0;
        return;
    }
    let Ok((player_position, player_sprite)) = player_query.get_single() else {
        return;
    };

    // If the player is already inside the location, reset the stuck time
    if roomdb
        .room_tiles
        .get(&player_position.to_board_position())
        .is_some()
    {
        *stuck_time = 0.0;
        walkie_play.mark(WalkieEvent::PlayerStuckAtStart, time.elapsed_secs_f64());
        return;
    }

    let distance_from_spawn = player_position.distance(&player_sprite.spawn_position);

    if distance_from_spawn < PLAYER_STUCK_MAX_DISTANCE {
        *stuck_time += time.delta_secs();
    } else {
        *stuck_time = 0.0;
        walkie_play.mark(WalkieEvent::PlayerStuckAtStart, time.elapsed_secs_f64());
    }

    if *stuck_time > PLAYER_STUCK_AT_START_SECONDS {
        warn!("Player stuck at start for {} seconds", *stuck_time);
        walkie_play.set(WalkieEvent::PlayerStuckAtStart, time.elapsed_secs_f64());
    }
}

fn check_erratic_movement_early(
    time: Res<Time>,
    game_state: Res<State<GameState>>,
    app_state: Res<State<AppState>>,
    roomdb: Res<RoomDB>,
    player_query: Query<(&Position, &Direction, &PlayerSprite)>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut not_entered_location_time: Local<f32>,
) {
    if app_state.get() != &AppState::InGame {
        *not_entered_location_time = 0.0;
        return;
    }
    if *game_state.get() != GameState::None {
        return;
    }

    let Ok((player_position, player_direction, player_sprite)) = player_query.get_single() else {
        return;
    };

    // Check if player is inside any room
    if roomdb
        .room_tiles
        .get(&player_position.to_board_position())
        .is_some()
    {
        *not_entered_location_time = 0.0;
        walkie_play.mark(WalkieEvent::ErraticMovementEarly, time.elapsed_secs_f64());
        return;
    }

    // If player is not in a room and in GameState::None, increment timer
    let distance_from_spawn = player_position.distance(&player_sprite.spawn_position);

    if distance_from_spawn > PLAYER_STUCK_MAX_DISTANCE
        && distance_from_spawn < PLAYER_ERRATIC_MAX_DISTANCE
        && player_direction.distance() > 2.0
    {
        // Ignore when the player is stuck or stopped.
        *not_entered_location_time += time.delta_secs();
    }

    if *not_entered_location_time > ERRATIC_MOVEMENT_EARLY_SECONDS {
        walkie_play.set(WalkieEvent::ErraticMovementEarly, time.elapsed_secs_f64());
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            crate::triggers::locomotion_interaction::check_player_stuck_at_start,
            crate::triggers::locomotion_interaction::check_erratic_movement_early,
        )
            .run_if(in_state(uncore::states::GameState::None)),
    );
}
