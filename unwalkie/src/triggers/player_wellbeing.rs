use bevy::app::App;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use uncore::components::light::LightLevel;
use uncore::{
    components::{
        board::position::Position,
        ghost_sprite::GhostSprite, // Added GhostSprite
        player::Hiding,            // Added Hiding
        player_sprite::PlayerSprite,
    },
    resources::{board_data::BoardData, roomdb::RoomDB}, // Added BoardData
    states::{AppState, GameState},
};
use unwalkiecore::{WalkieEvent, WalkiePlay}; // Corrected import for LightLevel

// Constants for SanityDroppedBelowThresholdDarkness
const LOW_LUX_THRESHOLD: f32 = 0.1;
const MIN_TIME_IN_DARKNESS_FOR_HINT_SECONDS: f32 = 45.0;
// SANITY_DROP_THRESHOLD_POINTS and MAX_SANITY_FOR_HINT_PERCENT are now shared
const SANITY_DROP_THRESHOLD_POINTS_SHARED: f32 = 15.0;
const MAX_SANITY_FOR_HINT_PERCENT_SHARED: f32 = 65.0;

// Constants for SanityDroppedBelowThresholdGhost
const GHOST_PROXIMITY_THRESHOLD: f32 = 3.0;
const MIN_INTERACTION_DURATION_SECONDS: f32 = 10.0;

/// Triggers a warning if the player's sanity drops below 30% and they don't return to the truck within 20 seconds.
fn very_low_sanity_no_truck_return(
    mut walkie_play: ResMut<WalkiePlay>,
    qp: Query<(&PlayerSprite, &Position)>,
    roomdb: Res<RoomDB>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut stopwatch: Local<Stopwatch>,
    time: Res<Time>,
) {
    if app_state.get() != &AppState::InGame || *game_state.get() != GameState::None {
        stopwatch.reset();
        return;
    }
    let Some((player, pos)) = qp.iter().next() else {
        return;
    };
    if player.sanity() >= 30.0 {
        stopwatch.reset();
        return;
    }
    let player_bpos = pos.to_board_position();
    if roomdb.room_tiles.get(&player_bpos).is_none() {
        // Player is not inside the location, reset timer
        stopwatch.reset();
        return;
    }
    stopwatch.tick(time.delta());
    if stopwatch.elapsed_secs() > 20.0 {
        // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
        walkie_play.set(
            WalkieEvent::VeryLowSanityNoTruckReturn,
            time.elapsed_secs_f64(),
        );
        stopwatch.reset();
    }
}

/// Triggers a warning if the player's health drops below 50% for 30 seconds while inside the location.
fn low_health_general_warning(
    mut walkie_play: ResMut<WalkiePlay>,
    qp: Query<(&PlayerSprite, &Position)>,
    roomdb: Res<RoomDB>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut stopwatch: Local<Stopwatch>,
    time: Res<Time>,
) {
    if app_state.get() != &AppState::InGame || *game_state.get() != GameState::None {
        stopwatch.reset();
        return;
    }
    let Some((player, pos)) = qp.iter().next() else {
        return;
    };
    if player.health >= 50.0 {
        stopwatch.reset();
        return;
    }
    let player_bpos = pos.to_board_position();
    if roomdb.room_tiles.get(&player_bpos).is_none() {
        // Player is not inside the location, reset timer
        stopwatch.reset();
        return;
    }
    stopwatch.tick(time.delta());
    if stopwatch.elapsed_secs() > 30.0 {
        // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
        walkie_play.set(
            WalkieEvent::LowHealthGeneralWarning,
            time.elapsed_secs_f64(),
        );
        stopwatch.reset();
    }
}

/// System for SanityDroppedBelowThresholdDarkness
fn trigger_sanity_dropped_due_to_darkness_system(
    time: Res<Time>,
    mut walkie_play: ResMut<WalkiePlay>,
    // FIXME: WTF is "LightLevel"? this does not exist, this seems a hallucination from the original code.
    player_query: Query<(&PlayerSprite, &Position, &LightLevel), Without<Hiding>>,
    roomdb: Res<RoomDB>,
    board_data: Res<BoardData>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut darkness_sanity_tracker: Local<Option<(f32, Stopwatch)>>, // (sanity_at_darkness_start, timer)
    mut hint_triggered_this_episode: Local<bool>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        *darkness_sanity_tracker = None;
        *hint_triggered_this_episode = false;
        return;
    }

    let Ok((player_sprite, player_pos, light_level)) = player_query.single() else {
        *darkness_sanity_tracker = None;
        *hint_triggered_this_episode = false;
        return;
    };

    // 2.b. Reset Conditions - Player not inside location or not in darkness
    let player_bpos = player_pos.to_board_position();
    if roomdb.room_tiles.get(&player_bpos).is_none() {
        *darkness_sanity_tracker = None;
        *hint_triggered_this_episode = false;
        return;
    }
    let is_in_darkness = light_level.lux < LOW_LUX_THRESHOLD && !board_data.is_lit(player_bpos);

    // 3.c. Defining "Prolonged Darkness Period"
    if is_in_darkness {
        match darkness_sanity_tracker.as_mut() {
            Some((_initial_sanity, timer)) => {
                timer.tick(time.delta());
            }
            None => {
                // Start timer only if player has been in darkness for a bit already
                // This avoids starting the timer for brief flickers into darkness.
                // For this, we'd need another timer or a way to check recent light history.
                // Simpler: just start the timer immediately when darkness is detected.
                // The MIN_TIME_IN_DARKNESS_FOR_HINT_SECONDS will gate the hint.
                // However, the prompt implies an "entry" duration. Let's use a simple proxy:
                // If they are in darkness now, and the timer is None, start it.
                // The PROLONGED_DARKNESS_ENTRY_SECONDS is not directly used to start the timer,
                // but rather the overall MIN_TIME_IN_DARKNESS_FOR_HINT_SECONDS implies a prolonged period.
                // Let's assume the intent is: if they enter darkness, start tracking. If that period
                // exceeds MIN_TIME_IN_DARKNESS_FOR_HINT_SECONDS, and other conditions met, fire.
                *darkness_sanity_tracker = Some((player_sprite.sanity(), Stopwatch::new()));
                *hint_triggered_this_episode = false; // Reset hint flag for new darkness episode
            }
        }
    } else {
        // Not in darkness
        *darkness_sanity_tracker = None;
        // *hint_triggered_this_episode = false; // Resetting here means if player briefly leaves darkness, hint can re-trigger.
    }

    // 3.e. Trigger Conditions
    if let Some((initial_sanity, timer)) = darkness_sanity_tracker.as_ref() {
        if *hint_triggered_this_episode {
            return;
        }
        // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
        if timer.elapsed_secs() >= MIN_TIME_IN_DARKNESS_FOR_HINT_SECONDS
            && player_sprite.sanity() < MAX_SANITY_FOR_HINT_PERCENT_SHARED
            && (*initial_sanity - player_sprite.sanity()) >= SANITY_DROP_THRESHOLD_POINTS_SHARED // Dereference initial_sanity
            && walkie_play.set(
                WalkieEvent::SanityDroppedBelowThresholdDarkness,
                time.elapsed_secs_f64(),
            )
        {
            *hint_triggered_this_episode = true;
        }
    }
}

// New System for SanityDroppedBelowThresholdGhost
fn trigger_sanity_dropped_due_to_ghost_system(
    time: Res<Time>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(&PlayerSprite, &Position, Option<&Hiding>)>,
    ghost_query: Query<(Entity, &GhostSprite, &Position)>, // Query Entity to track specific ghost
    roomdb: Res<RoomDB>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut interaction_sanity_tracker: Local<Option<(f32, Stopwatch, Entity)>>, // (sanity_at_interaction_start, timer, ghost_entity)
    mut hint_triggered_this_episode: Local<bool>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        *interaction_sanity_tracker = None;
        *hint_triggered_this_episode = false;
        return;
    }

    let Ok((player_sprite, player_pos, maybe_hiding)) = player_query.single() else {
        *interaction_sanity_tracker = None;
        *hint_triggered_this_episode = false;
        return;
    };

    // 2.b. Reset Conditions - Player not inside location or is hiding
    if roomdb
        .room_tiles
        .get(&player_pos.to_board_position())
        .is_none()
        || maybe_hiding.is_some()
    {
        *interaction_sanity_tracker = None;
        *hint_triggered_this_episode = false;
        return;
    }

    let player_bpos = player_pos.to_board_position();
    let player_room_name_opt = roomdb.room_tiles.get(&player_bpos);

    let mut current_interaction_ghost_entity: Option<Entity> = None;

    // 3.c. Defining "Ghost Interaction Period"
    if let Some(p_room_name) = player_room_name_opt {
        for (ghost_entity, ghost_sprite, ghost_pos) in ghost_query.iter() {
            // 3.c.i. Non-hunting ghost
            if ghost_sprite.hunting > 0.0 {
                // Consider if > 0.1 is better if hunting has a brief ramp-up
                continue;
            }
            // 3.c.iii. Player in same room
            let ghost_bpos = ghost_pos.to_board_position();
            if roomdb.room_tiles.get(&ghost_bpos) == Some(p_room_name)
            // Simplified map_or
            {
                // 3.c.ii. Player is close to this ghost
                if player_pos.distance(ghost_pos) < GHOST_PROXIMITY_THRESHOLD {
                    // Fallback visibility check: if ghost is in same room and not hunting, assume potentially visible.
                    // A more specific check like `ghost_sprite.is_manifested` or `ghost_sprite.alpha > 0.1` would be ideal.
                    // For now, proximity in same room for non-hunting ghost is the trigger.
                    current_interaction_ghost_entity = Some(ghost_entity);
                    break; // Interact with the first ghost that meets criteria
                }
            }
        }
    }

    // 3.d. Managing Interaction Timer
    if let Some(interacting_ghost_e) = current_interaction_ghost_entity {
        match interaction_sanity_tracker.as_mut() {
            Some((_initial_sanity, timer, tracked_e)) if *tracked_e == interacting_ghost_e => {
                // Still interacting with the same ghost, let timer tick
                timer.tick(time.delta());
            }
            _ => {
                // New interaction or different ghost
                *interaction_sanity_tracker = Some((
                    player_sprite.sanity(),
                    Stopwatch::new(),
                    interacting_ghost_e,
                ));
                *hint_triggered_this_episode = false; // Reset hint flag for new interaction episode
            }
        }
    } else {
        // No active interaction period
        *interaction_sanity_tracker = None;
        // *hint_triggered_this_episode = false; // Decided against resetting here to avoid re-trigger spam if player dips in/out of range
    }

    // 3.e. Trigger Conditions
    if let Some((initial_sanity, timer, _tracked_ghost_e)) = interaction_sanity_tracker.as_ref() {
        if *hint_triggered_this_episode {
            // If hint already fired for this specific interaction episode
            return;
        }
        // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
        if timer.elapsed_secs() >= MIN_INTERACTION_DURATION_SECONDS
            && player_sprite.sanity() < MAX_SANITY_FOR_HINT_PERCENT_SHARED
            && (*initial_sanity - player_sprite.sanity()) >= SANITY_DROP_THRESHOLD_POINTS_SHARED // Dereference initial_sanity
            && walkie_play.set(
                WalkieEvent::SanityDroppedBelowThresholdGhost,
                time.elapsed_secs_f64(),
            )
        {
            *hint_triggered_this_episode = true;
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, very_low_sanity_no_truck_return);
    app.add_systems(Update, low_health_general_warning);
    app.add_systems(Update, trigger_sanity_dropped_due_to_darkness_system); // Added new system
    app.add_systems(Update, trigger_sanity_dropped_due_to_ghost_system); // Added new system
}

// FIXME: The LightLevel component seems to be here as a placeholder, we need to understand its purpose.
// Dummy LightLevel component for compilation if not already defined elsewhere accessible
// #[derive(Component)]
// struct LightLevel {
//     lux: f32,
// }
