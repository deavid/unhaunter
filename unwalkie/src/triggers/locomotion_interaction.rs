use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_persistent::Persistent;

use uncore::behavior::component::Door;
use uncore::behavior::{Behavior, TileState};
use uncore::components::board::direction::Direction;
use uncore::components::board::position::Position;
use uncore::components::player::Hiding;
use uncore::components::player_sprite::PlayerSprite;
use uncore::resources::roomdb::RoomDB;
use uncore::states::{AppState, GameState};
use uncore::types::gear_kind::GearKind;
use ungear::components::playergear::PlayerGear;
use unprofile::PlayerProfileData;
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
    mut stuck_timer: Local<Stopwatch>,
    player_profile: Res<Persistent<PlayerProfileData>>,
) {
    let mut min_time_secs: f32 = 7.0;
    if app_state.get() != &AppState::InGame {
        stuck_timer.reset();
        return;
    }
    if *game_state.get() != GameState::None {
        stuck_timer.reset();
        return;
    }
    let Ok((player_position, player_sprite)) = player_query.get_single() else {
        return;
    };

    if player_profile.statistics.total_missions_completed > 1 {
        min_time_secs = 15.0;
    }
    if player_profile.statistics.total_missions_completed > 3 {
        min_time_secs = 30.0;
    }
    if player_profile.statistics.total_missions_completed > 6 {
        min_time_secs = 60.0;
    }
    if player_profile.statistics.total_missions_completed > 9 {
        min_time_secs = 90.0;
    }
    // If the player is already inside the location, reset the stuck time
    if roomdb
        .room_tiles
        .get(&player_position.to_board_position())
        .is_some()
    {
        stuck_timer.reset();
        walkie_play.mark(WalkieEvent::PlayerStuckAtStart, time.elapsed_secs_f64());
        return;
    }

    let distance_from_spawn = player_position.distance(&player_sprite.spawn_position);

    if distance_from_spawn < PLAYER_STUCK_MAX_DISTANCE {
        stuck_timer.tick(time.delta());
    } else {
        stuck_timer.reset();
        walkie_play.mark(WalkieEvent::PlayerStuckAtStart, time.elapsed_secs_f64());
    }

    if stuck_timer.elapsed_secs() > min_time_secs {
        // warn!("Player stuck at start for {} seconds", stuck_timer.elapsed_secs());
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
    mut not_entered_timer: Local<Stopwatch>,
    mut avg_position: Local<Option<Position>>,
    player_profile: Res<Persistent<PlayerProfileData>>,
) {
    if app_state.get() != &AppState::InGame {
        not_entered_timer.reset();
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
        not_entered_timer.reset();
        walkie_play.mark(WalkieEvent::ErraticMovementEarly, time.elapsed_secs_f64());
        return;
    }

    // If player is not in a room and in GameState::None, increment timer
    let distance_from_spawn = player_position.distance(&player_sprite.spawn_position);
    let distance_from_avg = player_position.distance(m_avg);

    if distance_from_avg > 3.0 {
        not_entered_timer.reset();
        return;
    }
    if distance_from_spawn > PLAYER_STUCK_MAX_DISTANCE
        && distance_from_spawn < PLAYER_ERRATIC_MAX_DISTANCE
        && player_direction.distance() > 60.0
    {
        // Ignore when the player is stuck or stopped.
        not_entered_timer.tick(time.delta());
    }

    if not_entered_timer.elapsed_secs() > ERRATIC_MOVEMENT_EARLY_SECONDS {
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
    mut hesitation_timer: Local<Stopwatch>,
) {
    if app_state.get() != &AppState::InGame {
        hesitation_timer.reset();
        return;
    }
    if *game_state.get() != GameState::None {
        hesitation_timer.reset();
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
        hesitation_timer.reset();
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
        hesitation_timer.reset();
        return;
    }

    hesitation_timer.tick(time.delta());

    if hesitation_timer.elapsed_secs() > 3.0
        && walkie_play.set(
            WalkieEvent::DoorInteractionHesitation,
            time.elapsed_secs_f64(),
        )
    {
        hesitation_timer.reset();
    }
}

/// Triggers a walkie-talkie event if the player attempts to grab an item when their inventory and hand are full,
/// and this state of attempting to grab while full persists.
fn trigger_struggling_with_grab_drop(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<(&PlayerGear, &PlayerSprite)>,
    mut full_and_failed_grab_timer: Local<Option<Stopwatch>>,
) {
    // 1. System Run Condition
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        *full_and_failed_grab_timer = None;
        return;
    }

    let Ok((player_gear, player_sprite)) = player_query.get_single() else {
        *full_and_failed_grab_timer = None;
        return;
    };
    if player_gear.held_item.is_some() {
        // If the player is already grabbing something, just skip this hint.
        *full_and_failed_grab_timer = None;
        return;
    }
    // 2.b. Check Player Full State
    let right_hand_full = !player_gear.empty_right_handed();
    let inventory_full = player_gear
        .as_vec()
        .iter()
        .all(|(g, _)| !matches!(g.kind, GearKind::None)); // Corrected to use GearKind::None
    let player_is_completely_full = right_hand_full && inventory_full;

    // 3.c. Detecting a Failed Grab Attempt to Start/Check Timer
    if keyboard_input.just_pressed(player_sprite.controls.grab) && player_is_completely_full {
        if full_and_failed_grab_timer.is_none() {
            *full_and_failed_grab_timer = Some(Stopwatch::new());
            // Timer starts, will be ticked below if it's Some.
        }
        // If timer was already Some (player pressed grab again while full and timer running), it just continues.
    } else if !player_is_completely_full && full_and_failed_grab_timer.is_some() {
        // Player is no longer full, so reset the timer.
        *full_and_failed_grab_timer = None;
    }

    // 3.d. Triggering Logic (if timer is Some)
    if let Some(ref mut timer_ref) = *full_and_failed_grab_timer {
        timer_ref.tick(time.delta()); // Tick the timer each frame it's Some

        // Re-check player_is_completely_full because they might have dropped/used an item
        // through a means other than the grab key (e.g., using a consumable from inventory directly)
        // which would not have reset the timer in the block above.
        let updated_right_hand_full = !player_gear.empty_right_handed();
        let updated_inventory_full = player_gear
            .inventory // Assuming inventory is the correct field name
            .iter()
            .all(|g| !matches!(g.kind, GearKind::None));
        let updated_player_is_completely_full = updated_right_hand_full && updated_inventory_full;

        if updated_player_is_completely_full {
            if timer_ref.elapsed_secs() > 5.0 {
                // Duration player struggles
                // FIXME: Additional verification and tuning is needed for this trigger. It worked before, but it was too much.
                if walkie_play.set(WalkieEvent::StrugglingWithGrabDrop, time.elapsed_secs_f64()) {
                    *full_and_failed_grab_timer = None; // Reset timer after successful trigger
                }
            }
        } else {
            // Player resolved the full state while timer was running
            *full_and_failed_grab_timer = None;
        }
    }
}

/// Triggers a walkie-talkie event if the player struggles to hide by holding [E] for over 2 seconds while not hidden.
///
/// This detects when a player holds the hide key ([E]) for over 2 seconds consecutively
/// without successfully hiding, indicating they're struggling because they're carrying a house item.
fn trigger_struggling_with_hide_unhide(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&PlayerSprite, Without<Hiding>>,
    mut hide_key_timer: Local<Option<Stopwatch>>,
) {
    if app_state.get() != &AppState::InGame {
        *hide_key_timer = None;
        return;
    }
    if *game_state.get() != GameState::None {
        *hide_key_timer = None;
        return;
    }

    // Only proceed if player is not hiding
    let Ok(player_sprite) = player_query.get_single() else {
        *hide_key_timer = None;
        return;
    };

    // Check if the hide key (activate key, typically [E]) is currently pressed
    let hide_key_pressed = keyboard_input.pressed(player_sprite.controls.activate);

    if hide_key_pressed {
        // Start or continue timer if key is pressed
        if hide_key_timer.is_none() {
            *hide_key_timer = Some(Stopwatch::new());
        }

        if let Some(ref mut timer) = *hide_key_timer {
            timer.tick(time.delta());

            // If player has been holding [E] for over 2 seconds while not hidden, trigger event
            if timer.elapsed_secs() > 2.0
                && walkie_play.set(
                    WalkieEvent::StrugglingWithHideUnhide,
                    time.elapsed_secs_f64(),
                )
            {
                // FIXME: Additional verification and tuning is needed for this trigger.
                // Reset timer after successful trigger to avoid spam
                *hide_key_timer = None;
            }
        }
    } else {
        // Reset timer if key is not pressed
        *hide_key_timer = None;
    }
}

/// Triggers a walkie-talkie event if the player stays hidden for too long after a hunt ends.
///
/// If the player remains hidden for 10+ seconds after the ghost's hunt ends, this event is triggered to inform them it's safe to unhide.
/// This event will not fire if any ghost's rage is above 20% of its rage limit.
fn trigger_player_stays_hidden_too_long(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    hiding_query: Query<Entity, With<Hiding>>,
    ghost_query: Query<&uncore::components::ghost_sprite::GhostSprite>,
    mut post_hunt_hidden_timer: Local<Option<f32>>,
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
    // Check if any ghost's rage is above 20% of its rage limit
    let high_rage = ghost_query.iter().any(|g| g.rage > g.rage_limit * 0.2);
    if high_rage {
        *post_hunt_hidden_timer = None;
        return;
    }
    // Player is hiding and hunt is over
    let now = time.elapsed_secs_f64() as f32;
    if let Some(start_time) = *post_hunt_hidden_timer {
        if now - start_time > 10.0 {
            // FIXME: Additional verification and tuning is needed for this trigger.
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
    player_query: Query<(&Position, Entity), Without<Hiding>>,
    hiding_spots: Query<(&Position, &Behavior)>,
    ghost_query: Query<&uncore::components::ghost_sprite::GhostSprite>,
    mut near_hiding_timer: Local<Option<f32>>,
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
                // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
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
            check_player_stuck_at_start,
            check_erratic_movement_early,
            check_door_interaction_hesitation,
            trigger_struggling_with_grab_drop, // This is the modified system
            trigger_struggling_with_hide_unhide,
            trigger_player_stays_hidden_too_long,
            trigger_hunt_active_near_hiding_spot_no_hide,
        )
            .run_if(in_state(GameState::None)), // Corrected in_state path
    );
}
