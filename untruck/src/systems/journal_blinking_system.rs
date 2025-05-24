use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_persistent::Persistent;
use uncore::{
    colors,
    components::truck_ui_button::TruckUIButton,
    resources::current_evidence_readings::CurrentEvidenceReadings,
    types::{
        evidence::Evidence,
        truck_button::{TruckButtonState, TruckButtonType},
    },
};
use unprofile::data::PlayerProfileData;
use unwalkiecore::resources::WalkiePlay;

pub const JOURNAL_HINT_THRESHOLD: u32 = 3;
pub const HIGH_CLARITY_THRESHOLD: f32 = 0.75;

#[allow(clippy::too_many_arguments)]
pub fn update_journal_button_blinking_system(
    walkie_play: Res<WalkiePlay>,
    current_evidence_readings: Res<CurrentEvidenceReadings>,
    profile_data: Res<Persistent<PlayerProfileData>>,
    mut button_query: Query<(&mut TruckUIButton, &mut BorderColor)>,
    time: Res<Time>,
    mut seen_evidence_hints: Local<HashSet<Evidence>>,
) {
    // Create a temporary map of evidence button states
    let mut evidence_button_states = bevy::utils::HashMap::new();
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
            seen_evidence_hints.insert(evidence_type);
        }
    }

    // Priority 2: High Clarity Unlogged (if no walkie target)
    if blinking_target_evidence.is_none() {
        for evidence_item in Evidence::all() {
            let clarity = current_evidence_readings
                .get_reading(evidence_item)
                .map_or(0.0, |reading| reading.clarity);

            if clarity >= HIGH_CLARITY_THRESHOLD {
                let ack_count = profile_data
                    .times_evidence_acknowledged_in_journal
                    .get(&evidence_item)
                    .copied()
                    .unwrap_or(0);

                // Maintain hint state if acknowledgment count is below threshold
                if ack_count < JOURNAL_HINT_THRESHOLD {
                    blinking_target_evidence = Some(evidence_item);
                    // Store this evidence in our persistent memory
                    seen_evidence_hints.insert(evidence_item);
                    break; // Found a high clarity target, no need to check further
                }
            }
        }
    }

    // Priority 3: Fallback to previously seen evidence hints (if no current target)
    if blinking_target_evidence.is_none() {
        // Filter seen evidence to only those not currently pressed and still under threshold
        for &evidence_item in seen_evidence_hints.iter() {
            let button_state = evidence_button_states
                .get(&evidence_item)
                .copied()
                .unwrap_or(TruckButtonState::Off);
            let ack_count = profile_data
                .times_evidence_acknowledged_in_journal
                .get(&evidence_item)
                .copied()
                .unwrap_or(0);

            // Only blink if button is not pressed and acknowledgment count is below threshold
            if button_state != TruckButtonState::Pressed && ack_count < JOURNAL_HINT_THRESHOLD {
                blinking_target_evidence = Some(evidence_item);
                break; // Found a persistent hint target
            }
        }
    }

    // TODO: Clear seen_evidence_hints when mission ends
    // This would require access to mission state or game state to detect mission exit

    // Apply Blinking to Buttons
    for (mut truck_button, mut border_color) in button_query.iter_mut() {
        if let TruckButtonType::Evidence(button_evidence_type) = truck_button.class {
            let should_have_hint = if let Some(target_ev) = blinking_target_evidence {
                target_ev == button_evidence_type
            } else {
                false
            };

            // Set the blinking hint state on the button
            truck_button.set_blinking_hint(should_have_hint);

            // Apply beautiful sine wave blinking animation
            // Only show visual blinking if hint is active AND button is not pressed
            let should_show_visual_blinking = should_have_hint
                && !truck_button.disabled
                && truck_button.status != TruckButtonState::Pressed;

            if should_show_visual_blinking {
                let pulse_factor =
                    (time.elapsed_secs_f64() * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5; // Varies 0.0 to 1.0
                let normal_color = truck_button.border_color(bevy::ui::Interaction::None);
                border_color.0 = normal_color.mix(
                    &colors::JOURNAL_BUTTON_BLINK_BORDER_COLOR,
                    pulse_factor as f32,
                );
            } else {
                // Reset to normal if not the target, disabled, or button is pressed
                border_color.0 = truck_button.border_color(bevy::ui::Interaction::None);
            }
        } else {
            // Ensure other (non-evidence) buttons are reset to their normal border color
            border_color.0 = truck_button.border_color(bevy::ui::Interaction::None);
        }
    }
}
