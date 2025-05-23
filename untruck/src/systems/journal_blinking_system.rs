use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::types::truck_button::TruckButtonState;
use uncore::{
    colors,
    components::truck_ui_button::TruckUIButton,
    resources::current_evidence_readings::CurrentEvidenceReadings,
    types::{evidence::Evidence, truck_button::TruckButtonType},
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
    mut button_query: Query<(&TruckUIButton, &mut BorderColor)>,
    time: Res<Time>,
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

        if ack_count < JOURNAL_HINT_THRESHOLD
            && evidence_button_states.get(&evidence_type) != Some(&TruckButtonState::Pressed)
        {
            blinking_target_evidence = Some(evidence_type);
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

                if ack_count < JOURNAL_HINT_THRESHOLD
                    && evidence_button_states.get(&evidence_item)
                        != Some(&TruckButtonState::Pressed)
                {
                    blinking_target_evidence = Some(evidence_item);
                    break; // Found a high clarity target, no need to check further
                }
            }
        }
    }

    // Apply Blinking to Buttons
    for (truck_button, mut border_color) in button_query.iter_mut() {
        let mut is_target_to_blink = false;
        if let TruckButtonType::Evidence(button_evidence_type) = truck_button.class {
            if let Some(target_ev) = blinking_target_evidence {
                if target_ev == button_evidence_type {
                    is_target_to_blink = true;
                }
            }

            if is_target_to_blink
                && !truck_button.disabled
                && truck_button.status != TruckButtonState::Pressed
            {
                let pulse_factor =
                    (time.elapsed_secs_f64() * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5; // Varies 0.0 to 1.0
                let normal_color = truck_button.border_color(bevy::ui::Interaction::None);
                border_color.0 = normal_color.mix(
                    &colors::JOURNAL_BUTTON_BLINK_BORDER_COLOR,
                    pulse_factor as f32,
                );
            } else {
                // Reset to normal if not the target, or disabled, or already pressed
                border_color.0 = truck_button.border_color(bevy::ui::Interaction::None);
            }
        } else {
            // Ensure other (non-evidence) buttons are reset to their normal border color
            // This case should ideally not be strictly necessary if only evidence buttons can have their border changed by this system,
            // but it's a good safeguard.
            border_color.0 = truck_button.border_color(bevy::ui::Interaction::None);
        }
    }
}
