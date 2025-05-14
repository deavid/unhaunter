use bevy::prelude::*;
use bevy_persistent::Persistent;

use uncore::behavior::component::Door;
use uncore::behavior::{Behavior, TileState};
use uncore::components::board::direction::Direction;
use uncore::components::{board::position::Position, player_sprite::PlayerSprite};
use uncore::resources::roomdb::RoomDB;
use uncore::states::{AppState, GameState};
use unprofile::data::PlayerProfileData;
use unwalkiecore::{WalkieEvent, WalkiePlay};

const PLAYER_STUCK_MAX_DISTANCE: f32 = 1.0;
const ERRATIC_MOVEMENT_EARLY_SECONDS: f32 = 5.0;
const PLAYER_ERRATIC_MAX_DISTANCE: f32 = 6.0;

fn check_player_stuck_at_start(
    time: Res<Time>,
    game_state: Res<State<GameState>>,
    app_state: Res<State<AppState>>,
    roomdb: Res<RoomDB>,
    player_query: Query<(&Position, &PlayerSprite)>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut stuck_time: Local<f32>,
    player_profile: Res<Persistent<PlayerProfileData>>,
) {
    let mut min_time_secs: f32 = 30.0;
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

    if player_profile.statistics.total_missions_completed > 3 {
        min_time_secs = 90.0;
    }
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

    if *stuck_time > min_time_secs {
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
    mut avg_position: Local<Option<Position>>,
    player_profile: Res<Persistent<PlayerProfileData>>,
) {
    if app_state.get() != &AppState::InGame {
        *not_entered_location_time = 0.0;
        *avg_position = None;
        return;
    }
    if *game_state.get() != GameState::None {
        return;
    }

    // Trigger only if completed missions are 3 or less
    if player_profile.statistics.total_missions_completed > 3 {
        return;
    }

    let Ok((player_position, player_direction, player_sprite)) = player_query.get_single() else {
        return;
    };

    let m_avg = avg_position.get_or_insert_with(|| *player_position);
    *m_avg = m_avg.lerp(player_position, 0.5 * time.delta_secs());

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
    let distance_from_avg = player_position.distance(m_avg);

    if distance_from_avg > 2.0 {
        *not_entered_location_time = 0.0;
        return;
    }

    if distance_from_spawn > PLAYER_STUCK_MAX_DISTANCE
        && distance_from_spawn < PLAYER_ERRATIC_MAX_DISTANCE
        && player_direction.distance() > 60.0
    {
        // Ignore when the player is stuck or stopped.
        *not_entered_location_time += time.delta_secs();
    }

    if *not_entered_location_time > ERRATIC_MOVEMENT_EARLY_SECONDS {
        walkie_play.set(WalkieEvent::ErraticMovementEarly, time.elapsed_secs_f64());
    }
}

fn check_door_interaction_hesitation(
    time: Res<Time>,
    game_state: Res<State<GameState>>,
    app_state: Res<State<AppState>>,
    roomdb: Res<RoomDB>,
    player_query: Query<(&Position, &PlayerSprite)>,
    door_query: Query<(&Position, &Behavior), With<Door>>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut hesitation_timer: Local<f32>,
) {
    if app_state.get() != &AppState::InGame {
        *hesitation_timer = 0.0;
        return;
    }
    if *game_state.get() != GameState::None {
        *hesitation_timer = 0.0;
        return;
    }

    let Ok((player_position, _)) = player_query.get_single() else {
        return;
    };

    // Check if the player is outside the location
    let is_outside = roomdb
        .room_tiles
        .get(&player_position.to_board_position())
        .is_none();

    if !is_outside {
        *hesitation_timer = 0.0;
        return;
    }

    // Find the closest door to the player
    let closest_door = door_query.iter().min_by(|(pos_a, _), (pos_b, _)| {
        player_position
            .distance(pos_a)
            .partial_cmp(&player_position.distance(pos_b))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let Some((door_position, door_behavior)) = closest_door else {
        return;
    };

    let distance_to_door = player_position.distance(door_position);
    if distance_to_door > 1.5 || door_behavior.state() != TileState::Closed {
        *hesitation_timer = 0.0;
        return;
    }

    *hesitation_timer += time.delta_secs();

    if *hesitation_timer > 10.0
        && walkie_play.set(
            WalkieEvent::DoorInteractionHesitation,
            time.elapsed_secs_f64(),
        )
    {
        *hesitation_timer = 0.0;
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            crate::triggers::locomotion_interaction::check_player_stuck_at_start,
            crate::triggers::locomotion_interaction::check_erratic_movement_early,
            crate::triggers::locomotion_interaction::check_door_interaction_hesitation,
        )
            .run_if(in_state(uncore::states::GameState::None)),
    );
}
