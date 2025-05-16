// In unwalkie/src/triggers/basic_gear_usage.rs

use bevy::prelude::*;
use uncore::{
    components::{board::position::Position, player_sprite::PlayerSprite},
    resources::roomdb::RoomDB,
    states::{AppState, GameState},
    types::{evidence::Evidence, gear_kind::GearKind},
};
use ungear::components::playergear::PlayerGear;
use unwalkiecore::{WalkieEvent, WalkiePlay}; // Core walkie types

// Local struct to track the state for this specific trigger
#[derive(Debug, PartialEq, Clone)] // Added Clone for easier assignment
struct RightHandGearStateTracker {
    gear_kind: GearKind,    // The kind of evidence-gathering gear in the right hand.
    inactive_duration: f32, // How long it's been in hand, inactive, and trigger key not pressed.
}

fn trigger_gear_selected_not_activated_system(
    // Bevy Resources
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    roomdb: Res<RoomDB>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut walkie_play: ResMut<WalkiePlay>,
    // Player Query
    player_query: Query<(&PlayerSprite, &PlayerGear, &Position)>,
    // Local state for this system
    mut tracker: Local<Option<RightHandGearStateTracker>>,
    // Potentially Res<BoardData> if we need to check if player is near ghost/breach later
) {
    // 1. Check Global Conditions (InGame, GameState::None, Player inside location)
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        if tracker.is_some() {
            // Only reset if there was a tracker
            *tracker = None;
        }
        return;
    }

    let Ok((player_sprite, player_gear, player_pos)) = player_query.get_single() else {
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    };

    if roomdb
        .room_tiles
        .get(&player_pos.to_board_position())
        .is_none()
    {
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    }

    // 2. Inspect Right-Hand Gear & Check if it's an Evidence Tool
    let right_hand_gear = &player_gear.right_hand;
    if right_hand_gear.kind == GearKind::None {
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    }

    if Evidence::try_from(&right_hand_gear.kind).is_err() {
        // Not an evidence-gathering tool (e.g., Flashlight, Quartz, Salt, Sage)
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    }

    let Some(gear_data) = right_hand_gear.data.as_ref() else {
        // Gear has no usable data (should not happen for non-None evidence gear)
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    };

    // 3. Check Gear State Conditions (Can be enabled AND is not currently enabled)
    // TODO (David): Ensure all activatable evidence-gathering gear (e.g., UVTorch, RedTorch, Videocam)
    // correctly implements `GearUsable::can_enable()`. For most, this will likely just be `fn can_enable(&self) -> bool { true }`
    // unless specific conditions like battery prevent activation.
    if !gear_data.can_enable() || gear_data.is_enabled() {
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    }

    // 4. Manage Tracker State & Timer
    let current_gear_kind = right_hand_gear.kind.clone(); // Clone here as it's cheap
    let mut reset_timer_this_frame = false;

    if keyboard_input.just_pressed(player_sprite.controls.trigger) {
        // [R] key
        reset_timer_this_frame = true;
    }

    match tracker.as_mut() {
        Some(current_tracker_mut) => {
            if current_tracker_mut.gear_kind != current_gear_kind || reset_timer_this_frame {
                // Gear changed, or player pressed [R] -> reset the tracker for the new/current gear.
                *current_tracker_mut = RightHandGearStateTracker {
                    gear_kind: current_gear_kind,
                    inactive_duration: 0.0,
                };
            } else {
                // Same gear, not enabled, trigger key not just pressed -> increment duration.
                current_tracker_mut.inactive_duration += time.delta_secs();
            }
        }
        None => {
            // No tracker existed, or it was reset. Initialize for the current gear.
            // Don't start timer if [R] was just pressed.
            if !reset_timer_this_frame {
                *tracker = Some(RightHandGearStateTracker {
                    gear_kind: current_gear_kind,
                    inactive_duration: 0.0,
                });
            }
        }
    }

    // 5. Check Duration and Trigger Event
    if let Some(current_tracker_ref) = tracker.as_ref() {
        // Use .as_ref() for read-only access
        const INACTIVITY_THRESHOLD_SECONDS: f32 = 15.0; // Configurable threshold
        if current_tracker_ref.inactive_duration >= INACTIVITY_THRESHOLD_SECONDS
            && walkie_play.set(
                WalkieEvent::GearSelectedNotActivated,
                time.elapsed_secs_f64(),
            )
        {
            // Event was successfully set to play. Reset local tracker to prevent immediate re-trigger for this instance.
            *tracker = None;
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, trigger_gear_selected_not_activated_system);
    // ... other systems for this module ...
}
