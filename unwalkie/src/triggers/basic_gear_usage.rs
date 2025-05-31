// In unwalkie/src/triggers/basic_gear_usage.rs

use bevy::prelude::*;
use uncore::{
    components::{
        board::{boardposition::BoardPosition, position::Position},
        ghost_sprite::GhostSprite,
        player_sprite::PlayerSprite,
    },
    difficulty::CurrentDifficulty,
    resources::{board_data::BoardData, roomdb::RoomDB},
    states::{AppState, GameState},
    types::{evidence::Evidence, gear_kind::GearKind, manual::ManualChapterIndex},
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
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    roomdb: Res<RoomDB>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(&PlayerSprite, &PlayerGear, &Position)>,
    mut tracker: Local<Option<RightHandGearStateTracker>>,
    mut r_triggered: Local<i32>,
) {
    // 1. Check Global Conditions (InGame, GameState::None, Player inside location)
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        if tracker.is_some() {
            // Only reset if there was a tracker
            *tracker = None;
            *r_triggered = 0;
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
    let current_gear_kind = right_hand_gear.kind;
    let mut reset_timer_this_frame = false;

    if keyboard_input.just_pressed(player_sprite.controls.trigger) {
        // [R] key
        reset_timer_this_frame = true;
        *r_triggered += 1;
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
        const INACTIVITY_THRESHOLD_SECONDS: f32 = 10.0;
        if current_tracker_ref.inactive_duration
            >= INACTIVITY_THRESHOLD_SECONDS * (1 + *r_triggered * 5) as f32
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

const HOTSPOT_DURATION_THRESHOLD: f32 = 30.0; // Seconds
const HOTSPOT_PROXIMITY_THRESHOLD: f32 = 5.0; // Game units for "near ghost"

#[derive(PartialEq, Clone, Debug)]
struct IneffectiveToolInHotspotTracker {
    tool_in_hand: GearKind, // Thermometer or EMFMeter
    duration_in_hotspot_with_ineffective_tool_active: f32,
}

fn trigger_did_not_switch_starting_gear_in_hotspot_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(&PlayerSprite, &PlayerGear, &Position)>,
    ghost_query: Query<(&GhostSprite, &Position)>, // GhostSprite for breach_pos, Position for live pos
    board_data: Res<BoardData>, // For actual ghost evidences & fallback breach_pos
    roomdb: Res<RoomDB>,
    difficulty: Res<CurrentDifficulty>,
    mut tracker: Local<Option<IneffectiveToolInHotspotTracker>>,
) {
    // 1. System Run Condition & Chapter Check
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    }
    let current_chapter_index = difficulty
        .0
        .tutorial_chapter
        .map(|c| c.index())
        .unwrap_or(usize::MAX);
    if current_chapter_index > ManualChapterIndex::Chapter2.index() {
        // Only for Chapter 1 & 2 (or non-tutorial)
        if tracker.is_some() {
            *tracker = None;
        } // Reset if chapter is too high
        return;
    }

    // 2. Get Player & Ghost Info
    let Ok((_player_sprite, player_gear, player_pos)) = player_query.get_single() else {
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    };
    // Ghost's current position (if available) and its definitive spawn_point (breach)
    let (ghost_spawn_bpos, current_ghost_live_pos_opt): (BoardPosition, Option<Position>) =
        match ghost_query.get_single() {
            Ok((gs, g_pos)) => (gs.spawn_point.clone(), Some(*g_pos)),
            Err(_) => (board_data.breach_pos.to_board_position(), None), // Fallback if no GhostSprite
        };

    // 3. Hotspot Check
    let player_bpos = player_pos.to_board_position();
    let player_room = roomdb.room_tiles.get(&player_bpos);
    let breach_room = roomdb.room_tiles.get(&ghost_spawn_bpos);

    let mut in_hotspot = false;
    if player_room.is_some() {
        if player_room == breach_room {
            // In breach room
            in_hotspot = true;
        }
        if let Some(ghost_live_pos) = current_ghost_live_pos_opt {
            if player_room == roomdb.room_tiles.get(&ghost_live_pos.to_board_position()) {
                // In live ghost's current room
                in_hotspot = true;
            }
        }
    }
    if !in_hotspot {
        // Proximity check if not in same room by definition
        if let Some(ghost_live_pos) = current_ghost_live_pos_opt {
            if player_pos.distance(&ghost_live_pos) < HOTSPOT_PROXIMITY_THRESHOLD {
                in_hotspot = true;
            }
        } else {
            // If no live ghost, check proximity to breach
            if player_pos.distance(&ghost_spawn_bpos.to_position_center())
                < HOTSPOT_PROXIMITY_THRESHOLD
            {
                in_hotspot = true;
            }
        }
    }

    if !in_hotspot {
        return;
    }

    // 4. Inspect Right-Hand Gear
    let current_tool_kind = player_gear.right_hand.kind;
    if current_tool_kind != GearKind::Thermometer && current_tool_kind != GearKind::EMFMeter {
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    }
    let Some(gear_data) = player_gear.right_hand.data.as_ref() else {
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    };
    if !gear_data.is_enabled() {
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    }

    // 5. Check Tool Effectiveness
    let evidence_from_current_tool = Evidence::try_from(&current_tool_kind).ok();
    let tool_is_ineffective =
        evidence_from_current_tool.is_none_or(|ev| !board_data.evidences.contains(&ev));

    if !tool_is_ineffective {
        // Tool *could* be useful for this ghost
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    }

    // 6. Check if Player Has the *Other* Starting Tool
    let other_tool_kind = if current_tool_kind == GearKind::Thermometer {
        GearKind::EMFMeter
    } else {
        GearKind::Thermometer
    };
    let player_has_other_tool = player_gear
        .as_vec()
        .iter()
        .any(|(gear, _epos)| gear.kind == other_tool_kind);

    if !player_has_other_tool {
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    }

    // 7. Manage Tracker & Trigger
    match tracker.as_mut() {
        Some(current_tracker_mut) => {
            if current_tracker_mut.tool_in_hand == current_tool_kind {
                current_tracker_mut.duration_in_hotspot_with_ineffective_tool_active +=
                    time.delta_secs();
            } else {
                // Tool in hand changed, reset tracker
                *current_tracker_mut = IneffectiveToolInHotspotTracker {
                    tool_in_hand: current_tool_kind,
                    duration_in_hotspot_with_ineffective_tool_active: 0.0,
                };
            }
        }
        None => {
            *tracker = Some(IneffectiveToolInHotspotTracker {
                tool_in_hand: current_tool_kind,
                duration_in_hotspot_with_ineffective_tool_active: 0.0,
            });
        }
    }

    if let Some(current_tracker_ref) = tracker.as_ref() {
        if current_tracker_ref.duration_in_hotspot_with_ineffective_tool_active
            > HOTSPOT_DURATION_THRESHOLD
            && walkie_play.set(
                WalkieEvent::DidNotSwitchStartingGearInHotspot,
                time.elapsed_secs_f64(),
            )
        {
            *tracker = None; // Reset after triggering
        }
    }
}

const MIN_CHAPTER_FOR_CYCLE_HINT: ManualChapterIndex = ManualChapterIndex::Chapter2;
const TOOL_ACTIVE_THRESHOLD_SECONDS: f32 = 90.0;
const Q_PRESS_INACTIVITY_THRESHOLD_SECONDS: f32 = 75.0;

#[derive(PartialEq, Clone, Debug, Default)]
struct GearCycleUsageTracker {
    right_hand_tool_active: Option<GearKind>,
    time_with_current_tool_continuously_active: f32,
    time_since_last_q_press: f32,
}

fn trigger_did_not_cycle_to_other_gear_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(&PlayerSprite, &PlayerGear, &Position)>,
    roomdb: Res<RoomDB>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    difficulty: Res<CurrentDifficulty>,
    ghost_query: Query<&GhostSprite>, // Add ghost query to check hunting state
    mut tracker: Local<GearCycleUsageTracker>, // No Option, always track
) {
    // 1. System Run Condition & Chapter Check
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        *tracker = GearCycleUsageTracker::default(); // Reset on state change
        return;
    }
    let current_chapter_index = difficulty
        .0
        .tutorial_chapter
        .map(|c| c.index())
        .unwrap_or(usize::MAX);
    if current_chapter_index < MIN_CHAPTER_FOR_CYCLE_HINT.index() {
        *tracker = GearCycleUsageTracker::default();
        return;
    }

    // 2. Get Player Info & Location Check
    let Ok((player_sprite, player_gear, player_pos)) = player_query.get_single() else {
        *tracker = GearCycleUsageTracker::default();
        return;
    };
    if roomdb
        .room_tiles
        .get(&player_pos.to_board_position())
        .is_none()
    {
        return;
    }

    // Check if any ghost is currently hunting - pause tracking if so
    let ghost_hunting = ghost_query.iter().any(|g| g.hunting > 0.0);
    if ghost_hunting {
        // Don't reset tracker, just return and pause tracking while ghost is hunting
        return;
    }

    // 3. Manage Tracker - time_since_last_q_press
    if keyboard_input.just_pressed(player_sprite.controls.cycle) {
        // [Q] key
        tracker.time_since_last_q_press = 0.0;
    } else {
        tracker.time_since_last_q_press += time.delta_secs();
    }

    // 4. Manage Tracker - time_with_current_tool_continuously_active
    let current_right_tool_kind = player_gear.right_hand.kind;
    let mut is_current_tool_an_active_evidence_tool = false;

    if current_right_tool_kind != GearKind::None
        && Evidence::try_from(&current_right_tool_kind).is_ok()
    {
        if let Some(gear_data) = player_gear.right_hand.data.as_ref() {
            if gear_data.is_enabled() {
                is_current_tool_an_active_evidence_tool = true;
            }
        }
    }

    if is_current_tool_an_active_evidence_tool {
        if tracker.right_hand_tool_active == Some(current_right_tool_kind) {
            tracker.time_with_current_tool_continuously_active += time.delta_secs();
        } else {
            // Tool changed or just became an active evidence tool
            tracker.right_hand_tool_active = Some(current_right_tool_kind);
            tracker.time_with_current_tool_continuously_active = 0.0;
        }
    } else {
        // No active evidence tool in right hand, or not an evidence tool
        tracker.right_hand_tool_active = None;
        tracker.time_with_current_tool_continuously_active = 0.0;
    }

    // 5. Check for Other Usable Evidence Tools
    if !is_current_tool_an_active_evidence_tool {
        // Only proceed if current tool is an active evidence one
        return;
    }

    let mut has_other_usable_evidence_tools = false;
    // Check left hand
    if player_gear.left_hand.kind != GearKind::None &&
       player_gear.left_hand.kind != current_right_tool_kind && // Different tool
       Evidence::try_from(&player_gear.left_hand.kind).is_ok()
    {
        if let Some(lh_data) = player_gear.left_hand.data.as_ref() {
            if lh_data.can_enable() {
                // Check if it *can* be enabled
                has_other_usable_evidence_tools = true;
            }
        }
    }
    // Check inventory if still no other tool found
    if !has_other_usable_evidence_tools {
        for gear_in_inv in &player_gear.inventory {
            if gear_in_inv.kind != GearKind::None &&
               gear_in_inv.kind != current_right_tool_kind && // Different tool
               Evidence::try_from(&gear_in_inv.kind).is_ok()
            {
                if let Some(inv_data) = gear_in_inv.data.as_ref() {
                    if inv_data.can_enable() {
                        has_other_usable_evidence_tools = true;
                        break;
                    }
                }
            }
        }
    }

    if !has_other_usable_evidence_tools {
        return; // No other distinct, usable evidence tools to cycle to
    }
    // 6. Trigger Condition
    // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
    if tracker.time_with_current_tool_continuously_active > TOOL_ACTIVE_THRESHOLD_SECONDS
        && tracker.time_since_last_q_press > Q_PRESS_INACTIVITY_THRESHOLD_SECONDS
        && walkie_play.set(WalkieEvent::DidNotCycleToOtherGear, time.elapsed_secs_f64())
    {
        // Reset timers after successfully triggering to give player time
        tracker.time_with_current_tool_continuously_active = 0.0;
        tracker.time_since_last_q_press = 0.0;
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, trigger_gear_selected_not_activated_system);
    app.add_systems(
        Update,
        trigger_did_not_switch_starting_gear_in_hotspot_system,
    );
    app.add_systems(Update, trigger_did_not_cycle_to_other_gear_system);
}
