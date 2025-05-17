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

/// Checks if the player is stuck at the spawn point for too long at the start of a mission.
///
/// If the player hasn't entered the location and remains near the spawn for a threshold duration,
/// triggers a walkie-talkie warning. The threshold is higher for experienced players.
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

/// Detects erratic movement patterns early in the mission for new players.
///
/// If the player moves back and forth near the spawn without entering the location for several seconds,
/// triggers a walkie-talkie warning. Only applies to players with few completed missions.
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

/// Checks if the player hesitates at a closed door near the entrance for too long.
///
/// If the player remains outside, close to a closed door, and doesn't interact for over 10 seconds,
/// triggers a walkie-talkie hint about door interaction.
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

/// Triggers a walkie-talkie event if the player tries to pick up an item but their right hand and inventory are full.
fn trigger_struggling_with_grab_drop(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<(&ungear::components::playergear::PlayerGear, &PlayerSprite)>,
    mut fail_timer: Local<f32>,
) {
    if app_state.get() != &AppState::InGame {
        *fail_timer = 0.0;
        return;
    }
    if *game_state.get() != GameState::None {
        *fail_timer = 0.0;
        return;
    }
    let Ok((player_gear, player_sprite)) = player_query.get_single() else {
        return;
    };
    // If right hand is not empty and all inventory slots are full, increment timer
    let right_full = !player_gear.empty_right_handed();
    let all_full = player_gear
        .inventory
        .iter()
        .all(|g| !matches!(g.kind, uncore::types::gear_kind::GearKind::None));
    if right_full && all_full {
        if keyboard_input.just_pressed(player_sprite.controls.grab) {
            *fail_timer += time.delta_secs();
        }
        if *fail_timer > 2.0 {
            walkie_play.set(WalkieEvent::StrugglingWithGrabDrop, time.elapsed_secs_f64());
            *fail_timer = 0.0;
        }
    } else {
        *fail_timer = 0.0;
    }
}

/// Triggers a walkie-talkie event if the player struggles with hiding/unhiding actions.
///
/// This includes:
/// - Trying to hide while carrying an item (right hand not empty)
/// - Unhiding immediately after hiding (within 2 seconds)
/// - Failing to hide during a hunt due to carrying an item
fn trigger_struggling_with_hide_unhide(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<
        (Entity, &ungear::components::playergear::PlayerGear),
        Without<uncore::components::player::Hiding>,
    >,
    hiding_query: Query<
        (Entity, &ungear::components::playergear::PlayerGear),
        With<uncore::components::player::Hiding>,
    >,
    ghost_query: Query<&uncore::components::ghost_sprite::GhostSprite>,
    mut last_hide_state: Local<Option<(bool, f32)>>, // (is_hiding, time_of_change)
) {
    if app_state.get() != &AppState::InGame {
        *last_hide_state = None;
        return;
    }
    if *game_state.get() != GameState::None {
        *last_hide_state = None;
        return;
    }
    // Determine if player is hiding
    let (is_hiding, player_gear) = if let Ok((_, gear)) = hiding_query.get_single() {
        (true, gear)
    } else if let Ok((_, gear)) = player_query.get_single() {
        (false, gear)
    } else {
        return;
    };
    let now = time.elapsed_secs_f64() as f32;

    // Track hide/unhide transitions
    if let Some((was_hiding, last_change_time)) = *last_hide_state {
        if !was_hiding && is_hiding {
            // Player just hid
            // If right hand is not empty, trigger event
            if !player_gear.empty_right_handed() {
                walkie_play.set(
                    WalkieEvent::StrugglingWithHideUnhide,
                    time.elapsed_secs_f64(),
                );
            }
            *last_hide_state = Some((true, now));
        } else if was_hiding && !is_hiding {
            // Player just unhid
            let hide_duration = now - last_change_time;
            if hide_duration < 2.0 {
                walkie_play.set(
                    WalkieEvent::StrugglingWithHideUnhide,
                    time.elapsed_secs_f64(),
                );
            }
            *last_hide_state = Some((false, now));
        }
        // else: no state change
    } else {
        *last_hide_state = Some((is_hiding, now));
    }

    // If a hunt is ongoing, and player is hiding with an item in hand, trigger event
    let hunt_active = ghost_query.iter().any(|g| g.hunting > 0.0);
    if hunt_active && is_hiding && !player_gear.empty_right_handed() {
        walkie_play.set(
            WalkieEvent::StrugglingWithHideUnhide,
            time.elapsed_secs_f64(),
        );
    }
}

/// Triggers a walkie-talkie event if the player stays hidden for too long after a hunt ends.
///
/// If the player remains hidden for 10+ seconds after the ghost's hunt ends, this event is triggered to inform them it's safe to unhide.
fn trigger_player_stays_hidden_too_long(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    hiding_query: Query<Entity, With<uncore::components::player::Hiding>>,
    ghost_query: Query<&uncore::components::ghost_sprite::GhostSprite>,
    mut post_hunt_hidden_timer: Local<Option<f32>>, // Time spent hidden after hunt ended
) {
    if app_state.get() != &AppState::InGame {
        *post_hunt_hidden_timer = None;
        return;
    }
    if *game_state.get() != GameState::None {
        *post_hunt_hidden_timer = None;
        return;
    }
    // Only proceed if player is hiding
    if hiding_query.get_single().is_err() {
        *post_hunt_hidden_timer = None;
        return;
    }
    // Check if any ghost is currently hunting
    let hunt_active = ghost_query.iter().any(|g| g.hunting > 0.0);
    if hunt_active {
        *post_hunt_hidden_timer = None;
        return;
    }
    // Player is hiding and hunt is over
    let now = time.elapsed_secs_f64() as f32;
    if let Some(start_time) = *post_hunt_hidden_timer {
        if now - start_time > 10.0 {
            walkie_play.set(
                WalkieEvent::PlayerStaysHiddenTooLong,
                time.elapsed_secs_f64(),
            );
            // Only trigger once per hiding session
            *post_hunt_hidden_timer = None;
        }
    } else {
        // Start timer when hunt ends and player is still hiding
        *post_hunt_hidden_timer = Some(now);
    }
}

/// Triggers a walkie-talkie event if the player is near a hiding spot during a hunt but does not hide.
/// Fires HuntActiveNearHidingSpotNoHide if ghost is hunting, player is not hiding, and a hiding spot is within 1.5 units for 2+ seconds.
fn trigger_hunt_active_near_hiding_spot_no_hide(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(&Position, Entity), Without<uncore::components::player::Hiding>>,
    hiding_spots: Query<(&Position, &Behavior)>,
    ghost_query: Query<&uncore::components::ghost_sprite::GhostSprite>,
    mut near_hiding_timer: Local<Option<f32>>, // Time spent near hiding spot during hunt
) {
    if app_state.get() != &AppState::InGame {
        *near_hiding_timer = None;
        return;
    }
    if *game_state.get() != GameState::None {
        *near_hiding_timer = None;
        return;
    }
    // Check if any ghost is actively hunting (hunting > 10.0)
    let hunt_active = ghost_query.iter().any(|g| g.hunting > 10.0);
    if !hunt_active {
        *near_hiding_timer = None;
        return;
    }
    // Get player position (not hiding)
    let Ok((player_pos, _)) = player_query.get_single() else {
        *near_hiding_timer = None;
        return;
    };
    // Find a hiding spot within 1.5 units
    let near_hiding = hiding_spots
        .iter()
        .filter(|(_, behavior)| behavior.p.object.hidingspot)
        .any(|(spot_pos, _)| player_pos.distance(spot_pos) < 1.5);
    if near_hiding {
        let now = time.elapsed_secs_f64() as f32;
        if let Some(start) = *near_hiding_timer {
            if now - start > 2.0 {
                walkie_play.set(
                    WalkieEvent::HuntActiveNearHidingSpotNoHide,
                    time.elapsed_secs_f64(),
                );
                // Only trigger once per hunt
                *near_hiding_timer = None;
            }
        } else {
            *near_hiding_timer = Some(now);
        }
    } else {
        *near_hiding_timer = None;
    }
}

/// Registers the locomotion and interaction systems to the Bevy app.
///
/// These systems monitor player movement and interaction patterns to provide hints or warnings via walkie-talkie.
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            crate::triggers::locomotion_interaction::check_player_stuck_at_start,
            crate::triggers::locomotion_interaction::check_erratic_movement_early,
            crate::triggers::locomotion_interaction::check_door_interaction_hesitation,
            crate::triggers::locomotion_interaction::trigger_struggling_with_grab_drop,
            crate::triggers::locomotion_interaction::trigger_struggling_with_hide_unhide,
            crate::triggers::locomotion_interaction::trigger_player_stays_hidden_too_long,
            crate::triggers::locomotion_interaction::trigger_hunt_active_near_hiding_spot_no_hide, // Register new system
        )
            .run_if(in_state(uncore::states::GameState::None)),
    );
}
