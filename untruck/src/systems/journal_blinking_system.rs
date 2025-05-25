use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_persistent::Persistent;
use uncore::components::ghost_sprite::GhostSprite;
use uncore::states::GameState;
use uncore::types::ghost::types::GhostType;
use uncore::{
    colors,
    components::truck_ui_button::TruckUIButton,
    events::loadlevel::LevelLoadedEvent,
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

// Define the new resource
#[derive(Resource, Default)]
pub struct SeenEvidenceHints(HashSet<Evidence>);

fn update_journal_button_blinking_system(
    walkie_play: Res<WalkiePlay>,
    current_evidence_readings: Res<CurrentEvidenceReadings>,
    profile_data: Res<Persistent<PlayerProfileData>>,
    mut button_query: Query<(&mut TruckUIButton, &mut BorderColor)>,
    time: Res<Time>,
    mut seen_evidence_hints: ResMut<SeenEvidenceHints>, // MODIFIED: Use ResMut
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
            seen_evidence_hints.0.insert(evidence_type); // MODIFIED: Access field of resource
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
                    seen_evidence_hints.0.insert(evidence_item); // MODIFIED: Access field of resource
                    break; // Found a high clarity target, no need to check further
                }
            }
        }
    }

    // Priority 3: Fallback to previously seen evidence hints (if no current target)
    if blinking_target_evidence.is_none() {
        // Filter seen evidence to only those not currently pressed and still under threshold
        for &evidence_item in seen_evidence_hints.0.iter() {
            // MODIFIED: Access field of resource
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

    // TODO: Clear seen_evidence_hints when mission ends - This will be handled by the new system

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
            } else if truck_button.blinking_hint_active {
                truck_button.blinking_hint_active = false;
                border_color.0 = truck_button.border_color(bevy::ui::Interaction::None);
            }
        }
    }
}

/// System for ghost button blinking when only one valid ghost candidate remains
fn update_journal_ghost_blinking_system(
    mut button_query: Query<(&mut TruckUIButton, &mut BorderColor)>,
    ghost_sprite_query: Query<&GhostSprite>,
    time: Res<Time>,
    seen_evidence_hints: Res<SeenEvidenceHints>,
) {
    // Get the actual mission ghost type from GhostSprite
    let actual_mission_ghost_type: Option<GhostType> =
        ghost_sprite_query.get_single().ok().map(|gs| gs.class);

    // Count enabled ghost buttons and track their states
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
    // Determine if we should blink:
    // 1. Seen evidence uniquely identifies the correct mission ghost.
    // 2. Exactly one enabled ghost button (not selected) matches this identified ghost.
    let mut blinking_target_ghost: Option<GhostType> = None;

    if let Some(correct_ghost_for_mission) = actual_mission_ghost_type {
        // For each enabled ghost, check if it's compatible with our seen evidence
        let mut compatible_ghosts = Vec::new();
        for &ghost_type in &visible_ghost_types {
            let ghost_evidences = ghost_type.evidences();
            // A ghost is compatible if all our seen evidence could be produced by this ghost
            let is_compatible = seen_evidence_hints
                .0
                .iter()
                .all(|&seen_ev| ghost_evidences.contains(&seen_ev));

            if is_compatible {
                compatible_ghosts.push(ghost_type);
            }
        }

        // Only proceed if evidence uniquely identifies the correct mission ghost among enabled options
        if compatible_ghosts.len() == 1 && compatible_ghosts[0] == correct_ghost_for_mission {
            // Now, apply original logic: only blink if one ghost button is enabled,
            // not pressed, and matches the (now confirmed by evidence) correct ghost.
            if enabled_ghost_buttons.len() == 1 {
                let (candidate_ghost, ghost_status) = enabled_ghost_buttons[0];

                if ghost_status != TruckButtonState::Pressed
                    && candidate_ghost == correct_ghost_for_mission
                {
                    blinking_target_ghost = Some(candidate_ghost);
                }
            }
        }
    }

    for (mut truck_button, mut border_color) in button_query.iter_mut() {
        if let TruckButtonType::Ghost(button_ghost_type) = truck_button.class {
            let should_have_hint = if let Some(target_ghost) = blinking_target_ghost {
                target_ghost == button_ghost_type
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

                // For ghost buttons, the normal border color is Color::NONE (transparent)
                // So we need to use a visible base color when blinking
                let base_color = match truck_button.class {
                    TruckButtonType::Ghost(_) => {
                        // Use a subtle base color instead of transparent for ghost buttons
                        colors::TRUCKUI_ACCENT2_COLOR
                    }
                    _ => truck_button.border_color(bevy::ui::Interaction::None),
                };

                let new_border_color = base_color.mix(
                    &colors::JOURNAL_BUTTON_BLINK_BORDER_COLOR,
                    pulse_factor as f32,
                );
                border_color.0 = new_border_color;
            } else if truck_button.blinking_hint_active {
                truck_button.blinking_hint_active = false;
                // Reset to normal if not the target, disabled, or button is pressed
                border_color.0 = truck_button.border_color(bevy::ui::Interaction::None);
            }
        }
    }
}

fn clear_seen_evidence_hints_on_mission_change(
    mut seen_evidence_hints: ResMut<SeenEvidenceHints>,
    mut level_loaded_events: EventReader<LevelLoadedEvent>,
) {
    // If any LevelLoadedEvent has occurred, it signifies a new level/mission has started.
    // We iterate through them to consume them for this reader and then clear the hints.
    for _event in level_loaded_events.read() {
        seen_evidence_hints.0.clear();
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<SeenEvidenceHints>();
    app.add_systems(Update, clear_seen_evidence_hints_on_mission_change);
    app.add_systems(
        FixedUpdate,
        (
            update_journal_button_blinking_system,
            update_journal_ghost_blinking_system,
        )
            .run_if(in_state(GameState::Truck)),
    );
}
