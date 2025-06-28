use super::uibutton::{TruckButtonState, TruckButtonType, TruckUIButton};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use bevy_platform::collections::HashSet;
use uncore::components::game_config::GameConfig;
use uncore::components::player_sprite::PlayerSprite;
use uncore::components::truck::TruckUIGhostGuess;
use uncore::difficulty::CurrentDifficulty;
use uncore::events::truck::TruckUIEvent;
use uncore::resources::ghost_guess::GhostGuess;
use uncore::resources::potential_id_timer::PotentialIDTimer;
use uncore::states::GameState;
use uncore::types::evidence::Evidence;
use uncore::types::ghost::types::GhostType;
use ungear::components::playergear::PlayerGear;
use unprofile::data::PlayerProfileData;
use unwalkiecore::resources::WalkiePlay;

/// Event to force discard an evidence type in the journal UI.
#[derive(Event, Debug, Clone, Copy)]
pub struct ForceDiscardEvidenceEvent(pub Evidence);

fn button_system(
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
    q_gear: Query<(&PlayerSprite, &mut PlayerGear)>,
    mut q_textcolor: Query<&mut TextColor>,
    mut gg: ResMut<GhostGuess>,
    mut ev_truckui: EventWriter<TruckUIEvent>,
    gc: Res<GameConfig>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut profile_data: ResMut<Persistent<PlayerProfileData>>,
    mut potential_id_timer: ResMut<PotentialIDTimer>,
    keyboard_input: Res<ButtonInput<KeyCode>>, // Add this resource
    difficulty: Res<CurrentDifficulty>,        // Add this parameter
    mut ev_force_discard: EventReader<ForceDiscardEvidenceEvent>,
) {
    // --- Handle ForceDiscardEvidenceEvents first ---
    for event in ev_force_discard.read() {
        println!(
            "Journal: Received ForceDiscardEvidenceEvent for {:?}",
            event.0
        );
        for (_, _, _, _, mut tui_button) in interaction_query.iter_mut() {
            if let TruckButtonType::Evidence(evidence_type) = tui_button.class {
                if evidence_type == event.0 {
                    println!(
                        "Journal: Setting evidence {:?} button to Discard",
                        evidence_type
                    );
                    tui_button.status = TruckButtonState::Discard;
                    // Potentially mark gg as changed if this directly affects it,
                    // or ensure downstream logic correctly picks up this button state change.
                }
            }
        }
    }

    let mut selected_evidences_found = HashSet::<Evidence>::new();
    let mut selected_evidences_missing = HashSet::<Evidence>::new();
    let mut clicked_ghost_type: Option<GhostType> = None;

    let shift_pressed =
        keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);

    // --- 1. GATHER INPUTS ---
    // First pass: Handle evidence button clicks and detect ghost button clicks.
    for (interaction, _, _, _, mut tui_button) in &mut interaction_query {
        // Skip buttons that use hold timer or are currently disabled from a previous frame
        if tui_button.disabled || tui_button.hold_duration.is_some() {
            continue;
        }

        if interaction.is_changed() && *interaction == Interaction::Pressed {
            match tui_button.class {
                TruckButtonType::Evidence(_) => {
                    if shift_pressed {
                        tui_button.toggle_discard();
                    } else {
                        tui_button.pressed();
                    }
                }
                TruckButtonType::Ghost(ghost_type) => {
                    if shift_pressed {
                        tui_button.toggle_discard();
                        if gg.ghost_type == Some(ghost_type) {
                            gg.ghost_type = None;
                        }
                    } else {
                        tui_button.pressed();
                        clicked_ghost_type = Some(ghost_type);
                    }
                }
                _ => {
                    // For Craft, End, etc.
                    if let Some(truckui_event) = tui_button.pressed() {
                        ev_truckui.write(truckui_event);
                    }
                }
            }
        }
    }

    // After handling clicks, now collect the final state of all evidence buttons
    for (_, _, _, _, tui_button) in &interaction_query {
        if let TruckButtonType::Evidence(evidence_type) = tui_button.class {
            match tui_button.status {
                TruckButtonState::Pressed => {
                    selected_evidences_found.insert(evidence_type);
                }
                TruckButtonState::Discard => {
                    selected_evidences_missing.insert(evidence_type);
                }
                _ => {}
            }
        }
    }

    // --- 2. UPDATE GHOSTGUESS RESOURCE ---
    let possible_ghosts: Vec<GhostType> = difficulty
        .0
        .ghost_set
        .as_vec()
        .into_iter()
        .filter(|ghost_type| {
            let ghost_ev = ghost_type.evidences();
            let mut is_discarded = false;
            for (_, _, _, _, tui_button) in &interaction_query {
                if let TruckButtonType::Ghost(gh) = tui_button.class {
                    if gh == *ghost_type && tui_button.status == TruckButtonState::Discard {
                        is_discarded = true;
                        break;
                    }
                }
            }
            !is_discarded
                && ghost_ev.is_superset(&selected_evidences_found)
                && ghost_ev.is_disjoint(&selected_evidences_missing)
        })
        .collect();

    // a) Handle manual click on a ghost button
    if let Some(clicked_ghost) = clicked_ghost_type {
        if gg.ghost_type == Some(clicked_ghost) {
            gg.ghost_type = None;
        } else {
            gg.ghost_type = Some(clicked_ghost);
        }
    }

    // b) Auto-deselect if the currently selected ghost becomes invalid
    if let Some(selected_ghost) = gg.ghost_type {
        if !possible_ghosts.contains(&selected_ghost) {
            gg.ghost_type = None;
        }
    }

    // c) Auto-select if only one ghost is possible and nothing is selected
    if possible_ghosts.len() == 1 && gg.ghost_type.is_none() {
        gg.ghost_type = Some(possible_ghosts[0]);
    }

    // --- 3. UPDATE UI FROM STATE ---
    // Second pass: Update visuals and disabled states of all buttons based on the now-finalized GhostGuess.
    for (interaction_ref, mut bgcolor, mut border_color, children, mut tui_button) in
        &mut interaction_query
    {
        let interaction = *interaction_ref;

        // Update ghost buttons' state and disabled status
        if let TruckButtonType::Ghost(gh) = tui_button.class {
            // --- MODIFIED LOGIC HERE ---
            // A ghost button is disabled if it's not a possible candidate,
            // UNLESS it is already in the Discard state (so it can be un-discarded).
            if tui_button.status == TruckButtonState::Discard {
                tui_button.disabled = false;
            } else {
                tui_button.disabled = !possible_ghosts.contains(&gh);
            }

            // The visual "Pressed" state is now purely based on GhostGuess.
            if gg.ghost_type == Some(gh) && tui_button.status != TruckButtonState::Discard {
                tui_button.status = TruckButtonState::Pressed;
            } else if tui_button.status != TruckButtonState::Discard {
                tui_button.status = TruckButtonState::Off;
            }
        }

        // --- NEW LOGIC FOR EVIDENCE BUTTONS ---
        if let TruckButtonType::Evidence(ev) = tui_button.class {
            // Primary check: is the gear for this evidence even available?
            if !difficulty
                .0
                .truck_gear
                .iter()
                .filter_map(|gear_kind| Evidence::try_from(gear_kind).ok())
                .collect::<HashSet<_>>()
                .contains(&ev)
            {
                tui_button.disabled = true;
            } else if tui_button.status == TruckButtonState::Off {
                // If gear is available, then apply the existing logic for 'Off' buttons
                let cannot_be_found = !possible_ghosts.is_empty()
                    && possible_ghosts.iter().all(|g| !g.evidences().contains(&ev));
                tui_button.disabled = cannot_be_found;
            } else {
                // If gear is available and button is already Pressed/Discard, it's never disabled.
                tui_button.disabled = false;
            }
        }

        // Update Craft Repellent button
        if let TruckButtonType::CraftRepellent = tui_button.class {
            let mut disabled = gg.ghost_type.is_none();
            if !disabled {
                for (player, gear) in q_gear.iter() {
                    if player.id == gc.player_id {
                        if let Some(ghost_type) = gg.ghost_type {
                            if !gear.can_craft_repellent(ghost_type) {
                                disabled = true;
                            }
                        }
                    }
                }
            }
            tui_button.disabled = disabled;
        }

        let current_interaction = if tui_button.disabled {
            Interaction::None
        } else {
            interaction
        };

        let mut textcolor = q_textcolor.get_mut(children[0]).unwrap();

        // Default color calculation
        let current_border_color = tui_button.border_color(current_interaction);
        let current_background_color = tui_button.background_color(current_interaction);
        let current_text_color = tui_button.text_color(current_interaction);

        if !tui_button.blinking_hint_active {
            border_color.0 = current_border_color;
        }
        *bgcolor = current_background_color.into();
        textcolor.0 = current_text_color;
    }

    // Update GhostGuess resource with the latest evidence sets (these might have changed)
    if gg.evidences_found != selected_evidences_found {
        println!(
            "Journal: Updating evidences_found: {:?} -> {:?}",
            gg.evidences_found, selected_evidences_found
        );
        gg.evidences_found = selected_evidences_found;
    }
    if gg.evidences_missing != selected_evidences_missing {
        println!(
            "Journal: Updating evidences_missing: {:?} -> {:?}",
            gg.evidences_missing, selected_evidences_missing
        );
        gg.evidences_missing = selected_evidences_missing;
    }

    // Acknowledge hints
    for (_interaction, _, _, _, tui_button) in &interaction_query {
        if let TruckButtonType::Evidence(clicked_evidence_type) = tui_button.class {
            if tui_button.status == TruckButtonState::Pressed {
                if let Some((hinted_evidence, _)) =
                    walkie_play.evidence_hinted_not_logged_via_walkie
                {
                    if hinted_evidence == clicked_evidence_type {
                        const JOURNAL_HINT_THRESHOLD: u32 = 3;
                        let ack_count = profile_data
                            .times_evidence_acknowledged_in_journal
                            .entry(clicked_evidence_type)
                            .or_insert(0);
                        if *ack_count < JOURNAL_HINT_THRESHOLD {
                            *ack_count += 1;
                            profile_data.set_changed();
                        }
                        walkie_play.clear_evidence_hint();
                    }
                }

                if let Some(potential_data) = &potential_id_timer.data {
                    if potential_data.evidence == clicked_evidence_type {
                        potential_id_timer.data = None;
                    }
                }
            }
        }
    }
}

fn ghost_guess_system(
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

pub(crate) fn app_setup(app: &mut App) {
    app.add_event::<ForceDiscardEvidenceEvent>()
        .add_systems(Update, ghost_guess_system)
        .add_systems(
            FixedUpdate,
            button_system.run_if(in_state(GameState::Truck)),
        );
}
