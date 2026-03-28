use crate::colors;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use bevy_platform::collections::{HashMap, HashSet};
use unghost_core::events::{EvidenceClarityThresholdCrossed, GhostActualTypeChanged};
use uninput_core::states::InGameUiState;
use uninvestigation_core::evidence::Evidence;
use uninvestigation_core::ghost::GhostType;
use unmapload_core::events::loadlevel::LevelLoadedEvent;
use unprofile_core::profile::PlayerProfileData;
use untruck_core::components::truck_ui_button::TruckUIButton;
use untruck_core::types::truck_button::{TruckButtonState, TruckButtonType};
use unwalkie_core::resources::WalkiePlay;

pub(crate) const JOURNAL_HINT_THRESHOLD: u32 = 3;

// Define the new resources
#[derive(Resource, Default)]
pub(crate) struct SeenEvidenceHints(pub(crate) HashSet<Evidence>);

/// Tracks which evidence types are currently above the high-clarity threshold.
/// Updated by `update_clarity_resource_from_events` from unghost-plugin events.
#[derive(Resource, Default)]
pub(crate) struct CurrentHighClarityEvidences(pub(crate) HashSet<Evidence>);

/// Caches the actual ghost type as emitted by unghost-plugin.
#[derive(Resource, Default)]
pub(crate) struct CachedGhostActualType(pub(crate) Option<GhostType>);

pub(crate) fn update_clarity_resource_from_events(
    mut ev_clarity: MessageReader<EvidenceClarityThresholdCrossed>,
    mut current_high: ResMut<CurrentHighClarityEvidences>,
    mut seen_hints: ResMut<SeenEvidenceHints>,
) {
    for ev in ev_clarity.read() {
        if ev.above_threshold {
            current_high.0.insert(ev.evidence);
            seen_hints.0.insert(ev.evidence);
        } else {
            current_high.0.remove(&ev.evidence);
        }
    }
}

pub(crate) fn cache_ghost_type_from_events(
    mut ev_ghost: MessageReader<GhostActualTypeChanged>,
    mut cached: ResMut<CachedGhostActualType>,
) {
    for ev in ev_ghost.read() {
        cached.0 = Some(ev.ghost_type);
    }
}

pub(crate) fn update_journal_button_blinking_system(
    walkie_play: Res<WalkiePlay>,
    current_high_clarity: Res<CurrentHighClarityEvidences>,
    profile_data: Res<Persistent<PlayerProfileData>>,
    mut button_query: Query<(&mut TruckUIButton, &mut BorderColor)>,
    time: Res<Time>,
    mut seen_evidence_hints: ResMut<SeenEvidenceHints>,
) {
    // Create a temporary map of evidence button states
    let mut evidence_button_states = HashMap::new();
    for (btn_config, _) in button_query.iter() {
        if let TruckButtonType::Evidence(ev) = btn_config.class {
            evidence_button_states.insert(ev, btn_config.status);
        }
    }

    let mut blinking_target_evidence: Option<Evidence> = None;

    // Priority 1: Walkie Prompt
    if let Some((evidence_type, _timestamp)) = walkie_play.evidence_hinted_not_logged_via_walkie {
        let ack_count = profile_data
            .times_evidence_acknowledged_in_journal
            .get(&evidence_type)
            .copied()
            .unwrap_or(0);

        // Maintain hint state if acknowledgment count is below threshold
        if ack_count < JOURNAL_HINT_THRESHOLD {
            blinking_target_evidence = Some(evidence_type);
            // Store this evidence in our persistent memory
            seen_evidence_hints.0.insert(evidence_type); // MODIFIED: Access field of resource
        }
    }

    // Priority 2: High Clarity Unlogged (if no walkie target)
    if blinking_target_evidence.is_none() {
        for evidence_item in Evidence::all() {
            if current_high_clarity.0.contains(&evidence_item) {
                let ack_count = profile_data
                    .times_evidence_acknowledged_in_journal
                    .get(&evidence_item)
                    .copied()
                    .unwrap_or(0);

                // Maintain hint state if acknowledgment count is below threshold
                if ack_count < JOURNAL_HINT_THRESHOLD {
                    blinking_target_evidence = Some(evidence_item);
                }
            }
        }
    }

    // Priority 3: Seen Hints (if no walkie or high clarity target)
    if blinking_target_evidence.is_none() {
        for evidence_item in seen_evidence_hints.0.iter() {
            let ack_count = profile_data
                .times_evidence_acknowledged_in_journal
                .get(evidence_item)
                .copied()
                .unwrap_or(0);

            if ack_count < JOURNAL_HINT_THRESHOLD {
                blinking_target_evidence = Some(*evidence_item);
                break;
            }
        }
    }

    // Apply blinking to evidence buttons
    let pulse_factor = (time.elapsed_secs_f64() * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5;

    for (mut truck_button, mut border_color) in button_query.iter_mut() {
        if let TruckButtonType::Evidence(ev) = truck_button.class {
            let should_have_hint = Some(ev) == blinking_target_evidence;
            truck_button.set_blinking_hint(should_have_hint);

            let should_show_visual_blinking = should_have_hint
                && !truck_button.disabled
                && truck_button.status != TruckButtonState::Pressed;

            if should_show_visual_blinking {
                let normal_color = {
                    let color = colors::TRUCKUI_ACCENT2_COLOR;
                    let alpha_disabled = if truck_button.disabled { 0.05 } else { 1.0 };
                    color.with_alpha(color.alpha() * alpha_disabled)
                };
                *border_color = BorderColor::all(normal_color.mix(
                    &colors::JOURNAL_BUTTON_BLINK_BORDER_COLOR,
                    pulse_factor as f32,
                ));
            }
        }
    }
}

pub(crate) fn update_journal_ghost_blinking_system(
    mut button_query: Query<(&mut TruckUIButton, &mut BorderColor)>,
    cached_ghost_type: Res<CachedGhostActualType>,
    time: Res<Time>,
    seen_evidence_hints: Res<SeenEvidenceHints>,
) {
    if cached_ghost_type.0.is_none() {
        debug!("update_journal_ghost_blinking_system: no ghost type cached yet");
    }
    let actual_mission_ghost_type: Option<GhostType> = cached_ghost_type.0;

    let mut enabled_ghost_buttons = Vec::new();
    let mut visible_ghost_types = Vec::new();

    for (btn_config, _) in button_query.iter() {
        if let TruckButtonType::Ghost(ghost_type) = btn_config.class {
            if !btn_config.disabled {
                enabled_ghost_buttons.push((ghost_type, btn_config.status));
            }
            visible_ghost_types.push(ghost_type);
        }
    }

    let mut blinking_target_ghost: Option<GhostType> = None;

    if let Some(correct_ghost_for_mission) = actual_mission_ghost_type {
        let mut compatible_ghosts = Vec::new();
        for &ghost_type in &visible_ghost_types {
            let ghost_evidences = ghost_type.evidences();
            let is_compatible = seen_evidence_hints
                .0
                .iter()
                .all(|&seen_ev| ghost_evidences.contains(&seen_ev));

            if is_compatible {
                compatible_ghosts.push(ghost_type);
            }
        }

        if compatible_ghosts.len() == 1
            && compatible_ghosts[0] == correct_ghost_for_mission
            && enabled_ghost_buttons.len() == 1
        {
            let (candidate_ghost, ghost_status) = enabled_ghost_buttons[0];

            if ghost_status != TruckButtonState::Pressed
                && candidate_ghost == correct_ghost_for_mission
            {
                blinking_target_ghost = Some(candidate_ghost);
            }
        }
    }

    let pulse_factor = (time.elapsed_secs_f64() * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5;

    for (mut truck_button, mut border_color) in button_query.iter_mut() {
        if let TruckButtonType::Ghost(button_ghost_type) = truck_button.class {
            let should_have_hint = blinking_target_ghost == Some(button_ghost_type);
            truck_button.set_blinking_hint(should_have_hint);

            let should_show_visual_blinking = should_have_hint
                && !truck_button.disabled
                && truck_button.status != TruckButtonState::Pressed;

            if should_show_visual_blinking {
                let base_color = colors::TRUCKUI_ACCENT2_COLOR;
                let new_border_color = base_color.mix(
                    &colors::JOURNAL_BUTTON_BLINK_BORDER_COLOR,
                    pulse_factor as f32,
                );
                *border_color = BorderColor::all(new_border_color);
            } else if truck_button.blinking_hint_active {
                truck_button.blinking_hint_active = false;
                let color = colors::TRUCKUI_ACCENT2_COLOR;
                let alpha_disabled = if truck_button.disabled { 0.05 } else { 1.0 };
                *border_color = BorderColor::all(color.with_alpha(color.alpha() * alpha_disabled));
            }
        }
    }
}

pub(crate) fn clear_seen_evidence_hints_on_mission_change(
    mut ev_level: MessageReader<LevelLoadedEvent>,
    mut seen_hints: ResMut<SeenEvidenceHints>,
    mut current_high: ResMut<CurrentHighClarityEvidences>,
    mut cached_ghost: ResMut<CachedGhostActualType>,
) {
    for _ in ev_level.read() {
        seen_hints.0.clear();
        current_high.0.clear();
        cached_ghost.0 = None;
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<SeenEvidenceHints>()
        .init_resource::<CurrentHighClarityEvidences>()
        .init_resource::<CachedGhostActualType>();

    app.add_systems(
        Update,
        (
            update_clarity_resource_from_events,
            cache_ghost_type_from_events,
            update_journal_button_blinking_system,
            update_journal_ghost_blinking_system,
        )
            .run_if(in_state(InGameUiState::Truck)),
    );

    app.add_systems(Update, clear_seen_evidence_hints_on_mission_change);
}
