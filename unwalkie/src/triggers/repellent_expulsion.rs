use bevy::prelude::*;
use std::any::Any;
use uncore::components::repellent_particle::RepellentParticle;
use uncore::resources::board_data::BoardData;
use uncore::types::gear_kind::GearKind;
use uncore::{
    components::{
        board::position::Position, ghost_sprite::GhostSprite, player_sprite::PlayerSprite,
    },
    resources::roomdb::RoomDB,
    states::{AppState, GameState},
};
use ungear::components::playergear::PlayerGear;
use ungearitems::components::repellentflask::RepellentFlask as RepellentFlaskData;
use unwalkiecore::{WalkieEvent, WalkiePlay};

/// How long player must linger after ghost is gone
const LINGER_THRESHOLD_SECONDS: f64 = 30.0;

fn trigger_ghost_expelled_player_lingers_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    ghost_query: Query<Entity, With<GhostSprite>>,
    player_query: Query<&Position, With<PlayerSprite>>, // Assuming only one player for now
    roomdb: Res<RoomDB>,
    mut ghost_gone_and_player_in_location_timestamp: Local<Option<f64>>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        // If not in the right state, reset the timer and do nothing
        if ghost_gone_and_player_in_location_timestamp.is_some() {
            *ghost_gone_and_player_in_location_timestamp = None;
        }
        return;
    }

    // 2. Check Ghost Presence
    let ghost_is_present = !ghost_query.is_empty();

    // 3. Check Player Location
    let Ok(player_pos) = player_query.get_single() else {
        // No player found, reset timer
        if ghost_gone_and_player_in_location_timestamp.is_some() {
            *ghost_gone_and_player_in_location_timestamp = None;
        }
        return;
    };
    let player_is_inside_location = roomdb
        .room_tiles
        .get(&player_pos.to_board_position())
        .is_some();

    // 4. Manage Timer and Trigger Logic
    if !ghost_is_present && player_is_inside_location {
        // Ghost is gone AND player is inside the location
        if ghost_gone_and_player_in_location_timestamp.is_none() {
            // Start the timer
            *ghost_gone_and_player_in_location_timestamp = Some(time.elapsed_secs_f64());
        } else if let Some(start_time) = *ghost_gone_and_player_in_location_timestamp {
            let duration_lingering = time.elapsed_secs_f64() - start_time;
            if duration_lingering > LINGER_THRESHOLD_SECONDS
                && walkie_play.set(
                    WalkieEvent::GhostExpelledPlayerLingers,
                    time.elapsed_secs_f64(),
                )
            {
                // Event successfully set, reset timer to prevent immediate re-trigger
                // for this "lingering session". The global WalkiePlay cooldown will handle mission-level frequency.
                *ghost_gone_and_player_in_location_timestamp = None;
            }
        }
    } else {
        // Ghost is present OR player is outside, reset the timer
        if ghost_gone_and_player_in_location_timestamp.is_some() {
            *ghost_gone_and_player_in_location_timestamp = None;
        }
    }
}

fn trigger_has_repellent_enters_location_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(&PlayerGear, &Position), With<PlayerSprite>>, // Removed PlayerSprite component as it's not directly used here
    roomdb: Res<RoomDB>,
    mut player_was_previously_outside: Local<bool>, // Tracks if player was outside in the last check
) {
    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        // If not in the right state, ensure the flag is reset for the next valid entry
        *player_was_previously_outside = true;
        return;
    }

    let Ok((player_gear, player_pos)) = player_query.get_single() else {
        // No player found
        *player_was_previously_outside = true; // Reset state
        return;
    };

    // 3. Check Repellent Status
    let has_valid_repellent = player_gear.as_vec().iter().any(|(gear, _epos)| {
        if gear.kind == GearKind::RepellentFlask {
            if let Some(rep_data_dyn) = gear.data.as_ref() {
                if let Some(rep_data) = <dyn Any>::downcast_ref::<RepellentFlaskData>(rep_data_dyn)
                {
                    return rep_data.liquid_content.is_some() && rep_data.qty > 0;
                }
            }
        }
        false
    });

    if !has_valid_repellent {
        // Player doesn't have a filled repellent, update flag and exit
        *player_was_previously_outside = roomdb
            .room_tiles
            .get(&player_pos.to_board_position())
            .is_none();
        return;
    }

    // 4. Determine Current Location Status
    let player_is_currently_inside = roomdb
        .room_tiles
        .get(&player_pos.to_board_position())
        .is_some();

    // 5. Detect Transition from Outside to Inside
    if player_is_currently_inside && *player_was_previously_outside {
        // Player just entered the location with a valid repellent
        walkie_play.set(
            WalkieEvent::HasRepellentEntersLocation,
            time.elapsed_secs_f64(),
        );
        // Note: The global WalkiePlay cooldown will manage re-triggering for this event.
        // No need to explicitly prevent re-triggering within this system beyond the state transition.
    }

    // 6. Update Previous Location Status for the next frame
    *player_was_previously_outside = !player_is_currently_inside;
}

const EFFECTIVE_REPELLENT_RANGE: f32 = 4.0; // Configurable distance in game units

// Local state to track if the repellent was active in the previous frame
#[derive(Default)]
struct PrevRepellentState {
    was_active: bool,
    // We might also store which flask (if player could have multiple, though unlikely now)
    // or the entity_id of the player to handle multiplayer later. For now, simple bool.
}

fn trigger_repellent_used_too_far_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(&PlayerGear, &Position), With<PlayerSprite>>,
    // Query for live ghost position
    ghost_pos_query: Query<&Position, (With<GhostSprite>, Without<PlayerSprite>)>,
    // Query GhostSprite component to get its spawn_point (breach_pos)
    ghost_sprite_query: Query<&GhostSprite, Without<PlayerSprite>>,
    board_data: Res<BoardData>, // Fallback for breach_pos if GhostSprite not available
    mut prev_repellent_state: Local<PrevRepellentState>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        prev_repellent_state.was_active = false; // Reset on state change
        return;
    }

    let Ok((player_gear, player_pos)) = player_query.get_single() else {
        prev_repellent_state.was_active = false;
        return;
    };

    // 2. Check current repellent state
    let mut current_repellent_is_active = false;
    if let Some(rep_flask_gear) = player_gear.as_vec().iter().find_map(|(g, _)| {
        if g.kind == GearKind::RepellentFlask {
            g.data.as_ref()
        } else {
            None
        }
    }) {
        if let Some(rep_data) = <dyn Any>::downcast_ref::<RepellentFlaskData>(rep_flask_gear) {
            current_repellent_is_active = rep_data.active && rep_data.qty > 0;
        }
    }

    // 3. Detect Activation: Was not active previously, but is active now.
    if current_repellent_is_active && !prev_repellent_state.was_active {
        // Repellent was just activated by the player this frame.

        // Determine target position for distance check
        let target_pos: Position = ghost_pos_query
            .get_single()
            .copied() // Use live ghost position if available
            .unwrap_or_else(|_| {
                // Fallback: use ghost's spawn_point (breach)
                ghost_sprite_query
                    .get_single()
                    .map(|gs| gs.spawn_point.to_position_center())
                    .unwrap_or_else(|_| board_data.breach_pos) // Final fallback
            });

        let distance = player_pos.distance(&target_pos);

        if distance > EFFECTIVE_REPELLENT_RANGE {
            walkie_play.set(WalkieEvent::RepellentUsedTooFar, time.elapsed_secs_f64());
        }
    }

    // 4. Update previous state for next frame
    prev_repellent_state.was_active = current_repellent_is_active;
}

const REACTION_WINDOW_SECONDS: f32 = 5.0;
const RAGE_SPIKE_THRESHOLD: f32 = 30.0; // How much rage must increase to be considered a spike
const PARTICLE_NEARBY_THRESHOLD: f32 = 3.5; // How close particles need to be to the ghost

#[derive(Default)]
struct RepellentReactionTracker {
    repellent_activated_time: f32,
    initial_ghost_rage: f32,
    initial_ghost_hunting_state: f32, // Using f32 to directly compare with GhostSprite.hunting
                                      // Potentially add ghost_entity_id if multiple ghosts were possible
}

// Local state to track if the repellent was active in the previous frame for activation detection
#[derive(Default)]
struct PrevRepellentActiveState {
    was_active: bool,
}

fn trigger_repellent_provokes_strong_reaction_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(&PlayerGear, &Position), With<PlayerSprite>>,
    mut ghost_query: Query<(&mut GhostSprite, &Position)>, // GhostSprite needs to be mutable if we were to add times_hunted
    repellent_particle_query: Query<&Position, With<RepellentParticle>>,
    mut tracker: Local<Option<RepellentReactionTracker>>,
    mut prev_rep_active_state: Local<PrevRepellentActiveState>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        *tracker = None;
        prev_rep_active_state.was_active = false;
        return;
    }

    let Ok((player_gear, _player_pos)) = player_query.get_single() else {
        *tracker = None;
        prev_rep_active_state.was_active = false;
        return;
    };
    let Ok((ghost_sprite, ghost_pos)) = ghost_query.get_single_mut() else {
        // Assuming one ghost
        *tracker = None;
        prev_rep_active_state.was_active = false;
        return;
    };

    // 2. Detect Player Repellent Activation
    let mut current_repellent_is_active_and_has_qty = false;
    if let Some(rep_flask_gear) = player_gear.as_vec().iter().find_map(|(g, _)| {
        if g.kind == GearKind::RepellentFlask {
            g.data.as_ref()
        } else {
            None
        }
    }) {
        if let Some(rep_data) = <dyn Any>::downcast_ref::<RepellentFlaskData>(rep_flask_gear) {
            current_repellent_is_active_and_has_qty = rep_data.active && rep_data.qty > 0;
        }
    }

    if current_repellent_is_active_and_has_qty && !prev_rep_active_state.was_active {
        // Repellent was just activated this frame by the player
        *tracker = Some(RepellentReactionTracker {
            repellent_activated_time: time.elapsed_secs(),
            initial_ghost_rage: ghost_sprite.rage,
            initial_ghost_hunting_state: ghost_sprite.hunting,
        });
    }
    prev_rep_active_state.was_active = current_repellent_is_active_and_has_qty;

    // 3. Monitor Ghost Reaction (if tracker is active)
    if let Some(tracker_data) = tracker.as_ref() {
        let time_since_activation = time.elapsed_secs() - tracker_data.repellent_activated_time;

        if time_since_activation <= REACTION_WINDOW_SECONDS {
            let rage_increase = ghost_sprite.rage - tracker_data.initial_ghost_rage;
            let hunt_just_started =
                ghost_sprite.hunting > 0.0 && tracker_data.initial_ghost_hunting_state == 0.0;
            // Also consider if hunt_warning_active just became true, if initial_ghost_hunting_state was low and warning was false
            let warning_just_started = ghost_sprite.hunt_warning_active
                && ghost_sprite.hunting < 1.0
                && tracker_data.initial_ghost_hunting_state < 1.0
                && ghost_sprite.rage > tracker_data.initial_ghost_rage;

            let particles_nearby = repellent_particle_query
                .iter()
                .any(|particle_pos| ghost_pos.distance(particle_pos) < PARTICLE_NEARBY_THRESHOLD);

            if (rage_increase > RAGE_SPIKE_THRESHOLD || hunt_just_started || warning_just_started)
                && particles_nearby
                && walkie_play.set(
                    WalkieEvent::RepellentUsedGhostEnragesPlayerFlees,
                    time.elapsed_secs_f64(),
                )
            {
                *tracker = None; // Reset tracker after successful trigger
            }
        } else {
            // Window has passed
            *tracker = None;
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, trigger_ghost_expelled_player_lingers_system);
    app.add_systems(Update, trigger_has_repellent_enters_location_system);
    app.add_systems(Update, trigger_repellent_provokes_strong_reaction_system);
    app.add_systems(Update, trigger_repellent_used_too_far_system);
    // ... other systems for this module ...
}
