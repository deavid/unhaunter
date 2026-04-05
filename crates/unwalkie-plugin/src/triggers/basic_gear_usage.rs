// In unwalkie/src/triggers/basic_gear_usage.rs

use bevy::prelude::*;
use unboard_core::resources::roomdb::RoomTopology;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use ungear_core::components::core::Battery;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::types::gear::kind::GearKind;
use unghost_core::resources::haunt_state::HauntState;
use unghost_core::resources::signals::GhostHuntSignals;
use uninput_core::components::PlayerInputMapping;
use uninteraction_core::interaction::Toggleable;
use uninvestigation_core::evidence::Evidence;
use unplayer_core::components::{MainPlayer, PlayerSprite};

use uncommon_states_core::UIContextState;
use unspatial_core::position::Position;
use unwalkie_core::events::walkie_types::WalkieEvent;
use unwalkie_core::resources::WalkiePlay; // Core walkie types

// Local struct to track the state for this specific trigger
#[derive(Debug, PartialEq, Clone)] // Added Clone for easier assignment
struct RightHandGearStateTracker {
    gear_kind: GearKind,    // The kind of evidence-gathering gear in the right hand.
    inactive_duration: f32, // How long it's been in hand, inactive, and trigger key not pressed.
}

fn trigger_gear_selected_not_activated_system(
    time: Res<Time>,
    app_state: Res<State<UIContextState>>,
    room_topology: Res<RoomTopology>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(&PlayerInputMapping, &PlayerGear, &Position), With<MainPlayer>>,
    q_gear: Query<(&GearKind, &Toggleable, Option<&Battery>)>,
    mut tracker: Local<Option<RightHandGearStateTracker>>,
    mut r_triggered: Local<i32>,
) {
    // 1. Check Global Conditions (InGame, Player inside location)
    if *app_state.get() != UIContextState::InGame {
        if tracker.is_some() {
            // Only reset if there was a tracker
            *tracker = None;
            *r_triggered = 0;
        }
        return;
    }

    let mut any_player_matches = false;
    let mut current_gear_kind = GearKind::None;
    let mut reset_timer_this_frame = false;

    for (input_mapping, player_gear, player_pos) in player_query.iter() {
        if room_topology
            .room_tiles
            .get(&player_pos.to_board_position())
            .is_none()
        {
            continue;
        }

        // 2. Inspect Right-Hand Gear & Check if it's an Evidence Tool
        let Some(right_hand_entity) = player_gear.right_hand else {
            continue;
        };

        let Ok((gear_kind, toggle, battery_opt)) = q_gear.get(right_hand_entity) else {
            continue;
        };

        if *gear_kind == GearKind::None {
            continue;
        }

        if Evidence::try_from(gear_kind).is_err() {
            // Not an evidence-gathering tool (e.g., Flashlight, Quartz, Salt, Sage)
            continue;
        }

        // 3. Check Gear State Conditions (Can be enabled AND is not currently enabled)
        let can_enable = if let Some(battery) = battery_opt {
            battery.level > 0.0
        } else {
            true
        };

        if !can_enable || toggle.is_on {
            continue;
        }

        any_player_matches = true;
        current_gear_kind = *gear_kind;

        if keyboard_input.just_pressed(input_mapping.controls.right_hand_trigger) {
            // [R] key
            reset_timer_this_frame = true;
            *r_triggered += 1;
        }
        break;
    }

    if !any_player_matches {
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    }

    // 4. Manage Tracker State & Timer
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // [R] key - This is redundant now but keeping it for context if needed,
        // actually r_triggered is already updated.
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
    app_state: Res<State<UIContextState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(&PlayerSprite, &PlayerGear, &Position), With<MainPlayer>>,
    hunt_signals: Res<GhostHuntSignals>,
    haunt_state: Res<HauntState>, // For actual ghost evidences & fallback breach_pos
    room_topology: Res<RoomTopology>,
    difficulty: Res<CurrentDifficulty>,
    q_gear: Query<(&GearKind, &Toggleable)>,
    mut tracker: Local<Option<IneffectiveToolInHotspotTracker>>,
) {
    // 1. System Run Condition & Chapter Check
    if *app_state.get() != UIContextState::InGame {
        if tracker.is_some() {
            *tracker = None;
        }
        return;
    }
    let current_chapter_index = difficulty.0.index();
    if current_chapter_index > undifficulty_core::difficulty::Difficulty::TutorialChapter2.index() {
        // Only for Chapter 1 & 2 (or non-tutorial)
        if tracker.is_some() {
            *tracker = None;
        } // Reset if chapter is too high
        return;
    }

    // 2. Get Ghost Info
    // Ghost's current position (if available) and its definitive spawn_point (breach)
    let mut ghost_targets = vec![];
    if let Some(primary) = hunt_signals.primary.clone() {
        ghost_targets.push((primary.spawn_point, Some(primary.position)));
    }
    if ghost_targets.is_empty() {
        ghost_targets.push((haunt_state.breach_pos.to_board_position(), None));
    }

    let mut any_in_hotspot_with_ineffective_tool = false;
    let mut current_tool_kind = GearKind::None;

    for (_player_sprite, player_gear, player_pos) in player_query.iter() {
        for (ghost_spawn_bpos, current_ghost_live_pos_opt) in &ghost_targets {
            // 3. Hotspot Check
            let player_bpos = player_pos.to_board_position();
            let player_room = room_topology.room_tiles.get(&player_bpos);
            let breach_room = room_topology.room_tiles.get(ghost_spawn_bpos);

            let mut in_hotspot = false;
            if player_room.is_some() {
                if player_room == breach_room {
                    // In breach room
                    in_hotspot = true;
                }
                if let Some(ghost_live_pos) = current_ghost_live_pos_opt
                    && player_room
                        == room_topology
                            .room_tiles
                            .get(&ghost_live_pos.to_board_position())
                {
                    // In live ghost's current room
                    in_hotspot = true;
                }
            }
            if !in_hotspot {
                // Proximity check if not in same room by definition
                if let Some(ghost_live_pos) = current_ghost_live_pos_opt {
                    if player_pos.distance(ghost_live_pos) < HOTSPOT_PROXIMITY_THRESHOLD {
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
                continue;
            }

            // 4. Inspect Right-Hand Gear
            let Some(right_hand_entity) = player_gear.right_hand else {
                continue;
            };
            let Ok((gear_kind, toggle)) = q_gear.get(right_hand_entity) else {
                continue;
            };
            let tool_kind = *gear_kind;

            if tool_kind != GearKind::Thermometer && tool_kind != GearKind::EMFMeter {
                continue;
            }
            if !toggle.is_on {
                continue;
            }

            // 5. Check Tool Effectiveness
            let evidence_from_current_tool = Evidence::try_from(&tool_kind).ok();
            let tool_is_ineffective =
                evidence_from_current_tool.is_none_or(|ev| !haunt_state.evidences.contains(&ev));

            if !tool_is_ineffective {
                // Tool *could* be useful for this ghost
                continue;
            }

            // 6. Check if Player Has the *Other* Starting Tool
            let other_tool_kind = if tool_kind == GearKind::Thermometer {
                GearKind::EMFMeter
            } else {
                GearKind::Thermometer
            };

            let mut player_has_other_tool = false;
            // Check left hand
            if let Some(e) = player_gear.left_hand
                && let Ok((k, _)) = q_gear.get(e)
                && *k == other_tool_kind
            {
                player_has_other_tool = true;
            }
            // Check inventory
            if !player_has_other_tool {
                for e in &player_gear.inventory {
                    if let Ok((k, _)) = q_gear.get(*e)
                        && *k == other_tool_kind
                    {
                        player_has_other_tool = true;
                        break;
                    }
                }
            }

            if player_has_other_tool {
                any_in_hotspot_with_ineffective_tool = true;
                current_tool_kind = tool_kind;
                break;
            }
        }
        if any_in_hotspot_with_ineffective_tool {
            break;
        }
    }

    if !any_in_hotspot_with_ineffective_tool {
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

    if let Some(current_tracker_ref) = tracker.as_ref()
        && current_tracker_ref.duration_in_hotspot_with_ineffective_tool_active
            > HOTSPOT_DURATION_THRESHOLD
        && walkie_play.set(
            WalkieEvent::DidNotSwitchStartingGearInHotspot,
            time.elapsed_secs_f64(),
        )
    {
        *tracker = None; // Reset after triggering
    }
}

const MIN_CHAPTER_FOR_CYCLE_HINT: usize = 1; // TutorialChapter2 is at index 1
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
    app_state: Res<State<UIContextState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(&PlayerInputMapping, &PlayerGear, &Position), With<MainPlayer>>,
    room_topology: Res<RoomTopology>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    difficulty: Res<CurrentDifficulty>,
    hunt_signals: Res<GhostHuntSignals>,
    q_gear: Query<(&GearKind, &Toggleable, Option<&Battery>)>,
    mut tracker: Local<GearCycleUsageTracker>, // No Option, always track
) {
    // 1. System Run Condition & Chapter Check
    if *app_state.get() != UIContextState::InGame {
        *tracker = GearCycleUsageTracker::default(); // Reset on state change
        return;
    }
    let current_chapter_index = difficulty.0.index();
    if current_chapter_index < MIN_CHAPTER_FOR_CYCLE_HINT {
        *tracker = GearCycleUsageTracker::default();
        return;
    }

    // 2. Get Player Info & Location Check
    let mut any_player_in_room = false;
    let mut matched_player_info: Option<(&PlayerInputMapping, &PlayerGear)> = None;

    for (input_mapping, player_gear, player_pos) in player_query.iter() {
        if room_topology
            .room_tiles
            .get(&player_pos.to_board_position())
            .is_some()
        {
            any_player_in_room = true;
            matched_player_info = Some((input_mapping, player_gear));
            break;
        }
    }

    if !any_player_in_room {
        *tracker = GearCycleUsageTracker::default();
        return;
    }

    let (input_mapping, player_gear) = matched_player_info.unwrap();

    // Check if any ghost is currently hunting - pause tracking if so
    if hunt_signals.any_hunting {
        // Don't reset tracker, just return and pause tracking while ghost is hunting
        return;
    }

    // 3. Manage Tracker - time_since_last_q_press
    if keyboard_input.just_pressed(input_mapping.controls.cycle) {
        // [Q] key
        tracker.time_since_last_q_press = 0.0;
    } else {
        tracker.time_since_last_q_press += time.delta_secs();
    }

    // 4. Manage Tracker - time_with_current_tool_continuously_active
    let mut current_right_tool_kind = GearKind::None;
    let mut is_current_tool_an_active_evidence_tool = false;

    if let Some(right_hand_entity) = player_gear.right_hand
        && let Ok((kind, toggle, _)) = q_gear.get(right_hand_entity)
    {
        current_right_tool_kind = *kind;
        if *kind != GearKind::None && Evidence::try_from(kind).is_ok() && toggle.is_on {
            is_current_tool_an_active_evidence_tool = true;
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

    let check_gear = |entity: Entity| -> bool {
        if let Ok((kind, _, battery_opt)) = q_gear.get(entity)
            && *kind != GearKind::None &&
               *kind != current_right_tool_kind && // Different tool
               Evidence::try_from(kind).is_ok()
        {
            // Check if it *can* be enabled
            if let Some(battery) = battery_opt {
                return battery.level > 0.0;
            }
            return true;
        }
        false
    };

    // Check left hand
    if let Some(e) = player_gear.left_hand
        && check_gear(e)
    {
        has_other_usable_evidence_tools = true;
    }
    // Check inventory if still no other tool found
    if !has_other_usable_evidence_tools {
        for e in &player_gear.inventory {
            if check_gear(*e) {
                has_other_usable_evidence_tools = true;
                break;
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
