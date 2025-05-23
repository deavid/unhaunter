use super::{
    GhostGuess, TruckUIEvent, TruckUIGhostGuess,
    uibutton::{TruckButtonState, TruckButtonType, TruckUIButton},
};
use bevy::{prelude::*, utils::HashSet};
use bevy_persistent::Persistent;
use uncore::components::game_config::GameConfig;
use uncore::components::player_sprite::PlayerSprite;
use uncore::resources::potential_id_timer::PotentialIDTimer; // New import
use uncore::types::evidence::Evidence;
use ungear::components::playergear::PlayerGear;
use unprofile::data::PlayerProfileData;
use unwalkiecore::resources::WalkiePlay;

#[allow(clippy::type_complexity)]
pub fn button_system(
    mut interaction_query: Query<
        (
            Ref<Interaction>,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
            &mut TruckUIButton,
        ),
        With<Button>,
    >,
    mut q_gear: Query<(&PlayerSprite, &mut PlayerGear)>,
    mut q_textcolor: Query<&mut TextColor>,
    mut gg: ResMut<GhostGuess>,
    mut ev_truckui: EventWriter<TruckUIEvent>,
    gc: Res<GameConfig>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut profile_data: ResMut<Persistent<PlayerProfileData>>,
    time: Res<Time>,
    mut potential_id_timer: ResMut<PotentialIDTimer>, // Added ResMut to allow modification
) {
    let mut selected_evidences_found = HashSet::<Evidence>::new();
    let mut selected_evidences_missing = HashSet::<Evidence>::new();
    let mut evidences_possible = HashSet::<Evidence>::new();
    let mut evidences_all_ghosts = HashSet::<Evidence>::new();
    let mut first_ghost = true;
    let mut new_ghost_selected = None;
    for (interaction, _color, _border_color, _children, mut tui_button) in &mut interaction_query {
        // Skip buttons that use hold timer - they're handled by the hold system
        if tui_button.hold_duration.is_some() {
            continue;
        }

        if let TruckButtonType::Evidence(evidence_type) = tui_button.class {
            match tui_button.status {
                TruckButtonState::Off => {}
                TruckButtonState::Pressed => {
                    selected_evidences_found.insert(evidence_type);
                }
                TruckButtonState::Discard => {
                    selected_evidences_missing.insert(evidence_type);
                }
            }
        }
        if tui_button.disabled {
            continue;
        }
        if interaction.is_changed() && *interaction == Interaction::Pressed {
            if let Some(truckui_event) = tui_button.pressed() {
                ev_truckui.send(truckui_event.clone());

                // Reset highlight_craft_button if CraftRepellent is clicked
                if let TruckButtonType::CraftRepellent = tui_button.class {
                    if walkie_play.highlight_craft_button {
                        walkie_play.highlight_craft_button = false;
                        // info!("Craft Repellent button clicked, highlight_craft_button reset.");
                    }
                }
            }

            // Acknowledgement Logic Start
            if let TruckButtonType::Evidence(clicked_evidence_type) = tui_button.class {
                // Check if this evidence was hinted by a walkie talkie message
                let mut acknowledged_walkie_hint = false;
                if let Some((hinted_evidence, _timestamp)) =
                    walkie_play.evidence_hinted_not_logged_via_walkie
                {
                    if hinted_evidence == clicked_evidence_type
                        && tui_button.status == TruckButtonState::Pressed
                    {
                        const JOURNAL_HINT_THRESHOLD: u32 = 3; // Consistent with blinking system
                        let ack_count_entry = profile_data
                            .times_evidence_acknowledged_in_journal
                            .entry(clicked_evidence_type)
                            .or_insert(0);

                        if *ack_count_entry < JOURNAL_HINT_THRESHOLD {
                            *ack_count_entry += 1;
                            profile_data.set_changed(); // Mark Persistent data as changed
                            // info!("Journal hint for {:?} acknowledged via walkie. New count: {}", clicked_evidence_type, *ack_count_entry);
                        }
                        acknowledged_walkie_hint = true;
                    }
                }

                if acknowledged_walkie_hint {
                    walkie_play.evidence_hinted_not_logged_via_walkie = None;
                }

                // Cancel PotentialIDTimer if the logged evidence matches
                if tui_button.status == TruckButtonState::Pressed {
                    if let Some((timed_evidence, _, _, _)) = potential_id_timer.data {
                        if timed_evidence == clicked_evidence_type {
                            // info!(
                            //     "Evidence {:?} logged in journal, cancelling PotentialIDTimer.",
                            //     clicked_evidence_type
                            // );
                            potential_id_timer.data = None;
                        }
                    }
                }
            }
            // Acknowledgement Logic End
        }
        if let TruckButtonType::Ghost(ghost_type) = tui_button.class {
            if interaction.is_changed() && tui_button.status == TruckButtonState::Pressed {
                new_ghost_selected = Some(ghost_type);
            }
            if tui_button.status != TruckButtonState::Discard {
                let gh_evidences = ghost_type.evidences();
                if first_ghost {
                    first_ghost = false;
                    evidences_all_ghosts.clone_from(&gh_evidences);
                } else {
                    let missing = evidences_all_ghosts
                        .difference(&gh_evidences)
                        .cloned()
                        .collect::<Vec<_>>();
                    for m_ev in missing {
                        evidences_all_ghosts.remove(&m_ev);
                    }
                }
                for evidence in gh_evidences {
                    evidences_possible.insert(evidence);
                }
            }
        }
    }
    let mut ghost_selected = None;
    for (interaction, mut color, mut border_color, children, mut tui_button) in
        &mut interaction_query
    {
        let pressed = tui_button.status == TruckButtonState::Pressed;
        if let TruckButtonType::CraftRepellent = tui_button.class {
            let mut disabled = gg.ghost_type.is_none();
            for (player, gear) in q_gear.iter_mut() {
                if player.id == gc.player_id {
                    if let Some(ghost_type) = gg.ghost_type {
                        if !gear.can_craft_repellent(ghost_type) {
                            disabled = true;
                        }
                    }
                }
            }
            if tui_button.disabled != disabled {
                tui_button.disabled = disabled;
            }
        }
        if let TruckButtonType::Evidence(ev) = tui_button.class {
            tui_button.disabled = (!evidences_possible.contains(&ev)
                && tui_button.status != TruckButtonState::Discard)
                || (evidences_all_ghosts.contains(&ev)
                    && tui_button.status == TruckButtonState::Off);
        }
        if let TruckButtonType::Ghost(gh) = tui_button.class {
            let ghost_ev = gh.evidences();
            let selected_ev_count = ghost_ev.intersection(&selected_evidences_found).count();
            let missing_ev_count = ghost_ev.intersection(&selected_evidences_missing).count();
            tui_button.disabled =
                selected_ev_count < selected_evidences_found.len() || missing_ev_count > 0;
            if let Some(ghost) = new_ghost_selected.as_ref() {
                let is_this_ghost = gh == *ghost;
                if !is_this_ghost && pressed {
                    tui_button.status = TruckButtonState::Off;
                }
            }
            if tui_button.status == TruckButtonState::Pressed {
                ghost_selected = Some(gh);
            }
        }
        let interaction = if tui_button.disabled {
            // Disable mouse actions when button is disabled
            Interaction::None
        } else {
            *interaction
        };
        let mut textcolor = q_textcolor.get_mut(children[0]).unwrap();

        // Default color calculation
        let mut current_border_color = tui_button.border_color(interaction);
        let current_background_color = tui_button.background_color(interaction);
        let current_text_color = tui_button.text_color(interaction);

        // UI Cue for Craft Repellent Button
        if let TruckButtonType::CraftRepellent = tui_button.class {
            if walkie_play.highlight_craft_button && !tui_button.disabled {
                let pulse_factor =
                    (time.elapsed_secs_f64() * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5; // Varies 0.0 to 1.0

                current_border_color = current_border_color.mix(
                    &uncore::colors::JOURNAL_BUTTON_BLINK_BORDER_COLOR, // Reusing journal blink color
                    pulse_factor as f32,
                );
                // Example for background, if desired:
                // current_background_color = current_background_color.mix(
                //     &uncore::colors::TRUCKUI_ACCENT2_COLOR, // Example: A slightly different shade for bg
                //     pulse_factor as f32
                // );
            }
        }

        border_color.0 = current_border_color;
        *color = current_background_color.into();
        textcolor.0 = current_text_color;
    }
    gg.ghost_type = ghost_selected;

    // Update GhostGuess with the latest selected evidences
    if gg.evidences_found != selected_evidences_found {
        gg.evidences_found = selected_evidences_found;
    }
    if gg.evidences_missing != selected_evidences_missing {
        gg.evidences_missing = selected_evidences_missing;
    }
}

pub fn ghost_guess_system(
    mut guess_query: Query<&mut Text, With<TruckUIGhostGuess>>,
    gg: Res<GhostGuess>,
) {
    if !gg.is_changed() {
        return;
    }
    for mut text in guess_query.iter_mut() {
        text.0 = match gg.ghost_type.as_ref() {
            Some(gn) => gn.name().to_owned(),
            None => "-- Unknown --".to_string(),
        };
    }
}
