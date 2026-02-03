use super::uibutton::{TruckButtonState, TruckButtonType, TruckUIButton};
use crate::components::truck::TruckUIGhostGuess;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unevents_core::events::truck::TruckUIEvent;
use unghost_core::resources::ghost_guess::GhostGuess;
use unghost_core::resources::potential_id_timer::PotentialIDTimer;
use unghost_core::types::evidence::Evidence;
use unghost_core::types::ghost::types::GhostType;
use unnet_core::messages::{
    JournalUpdateMsg, NetworkDataEvent, NetworkMessage, RequestJournalEvidenceToggleMsg,
    RequestJournalGhostToggleMsg, SendNetworkMessage,
};
use unnet_core::resources::LocalPlayer;
use unprofile_core::profile::PlayerProfileData;
use untruck_core::journal::ForceDiscardEvidenceEvent;
use untypes_core::cli::{CliOptions, NetMode};
use untypes_core::states::AppState;
use unwalkie_core::resources::WalkiePlay;

/// System that handles ForceDiscardEvidenceEvents even when not in truck
fn force_discard_evidence_system(
    mut interaction_query: Query<&mut TruckUIButton, With<Button>>,
    mut ev_force_discard: MessageReader<ForceDiscardEvidenceEvent>,
    mut gg: ResMut<GhostGuess>,
) {
    for event in ev_force_discard.read() {
        debug!(
            "Journal: Received ForceDiscardEvidenceEvent for {:?}",
            event.0
        );

        let mut button_found = false;
        for mut tui_button in interaction_query.iter_mut() {
            if let TruckButtonType::Evidence(evidence_type) = tui_button.class
                && evidence_type == event.0
            {
                debug!(
                    "Journal: Setting evidence {:?} button from {:?} to Discard",
                    evidence_type, tui_button.status
                );
                tui_button.status = TruckButtonState::Discard;
                tui_button.computer_locked = true;
                button_found = true;
                break;
            }
        }

        if button_found {
            // Force mark the GhostGuess as changed to trigger update systems
            gg.set_changed();
            debug!(
                "Journal: ForceDiscardEvidenceEvent processed for {:?}",
                event.0
            );
        } else {
            warn!("Journal: Could not find evidence button for {:?}", event.0);
        }
    }
}

#[derive(SystemParam)]
struct JournalButtonParams<'w, 's> {
    interaction_query: Query<
        'w,
        's,
        (
            Ref<'static, Interaction>,
            &'static mut BackgroundColor,
            &'static mut BorderColor,
            &'static Children,
            &'static mut TruckUIButton,
        ),
        With<Button>,
    >,
    q_textcolor: Query<'w, 's, &'static mut TextColor>,
    gg: ResMut<'w, GhostGuess>,
    ev_truckui: MessageWriter<'w, TruckUIEvent>,
    walkie_play: ResMut<'w, WalkiePlay>,
    profile_data: ResMut<'w, Persistent<PlayerProfileData>>,
    potential_id_timer: ResMut<'w, PotentialIDTimer>,
    keyboard_input: Res<'w, ButtonInput<KeyCode>>,
    difficulty: Res<'w, CurrentDifficulty>,
    cli: Res<'w, CliOptions>,
    ev_net: MessageWriter<'w, SendNetworkMessage>,
    local_id: Res<'w, LocalPlayer>,
}

fn button_system(mut p: JournalButtonParams) {
    let mut clicked_ghost_type: Option<(GhostType, bool)> = None;
    let mut clicked_evidence_type: Option<(Evidence, bool)> = None;

    let shift_pressed = p.keyboard_input.pressed(KeyCode::ShiftLeft)
        || p.keyboard_input.pressed(KeyCode::ShiftRight);

    // --- 1. GATHER INPUTS ---
    for (interaction, _, _, _, mut tui_button) in &mut p.interaction_query {
        if tui_button.disabled || tui_button.hold_duration.is_some() || tui_button.computer_locked {
            continue;
        }

        if interaction.is_changed() && *interaction == Interaction::Pressed {
            match tui_button.class {
                TruckButtonType::Evidence(evidence_type) => {
                    clicked_evidence_type = Some((evidence_type, shift_pressed));
                }
                TruckButtonType::Ghost(ghost_type) => {
                    clicked_ghost_type = Some((ghost_type, shift_pressed));
                }
                _ => {
                    // For Craft, End, etc.
                    // These are not (yet) handled via network messages.
                    if let Some(truckui_event) = tui_button.pressed() {
                        p.ev_truckui.write(truckui_event);
                    }
                }
            }
        }
    }

    // --- 2. UPDATE STATE (HOST) OR SEND MESSAGES (CLIENT) ---
    match p.cli.net_mode {
        NetMode::Offline | NetMode::Host { .. } => {
            if let Some((ev, discard)) = clicked_evidence_type {
                if discard {
                    if p.gg.evidences_missing.contains(&ev) {
                        p.gg.evidences_missing.remove(&ev);
                    } else {
                        p.gg.evidences_missing.insert(ev);
                        p.gg.evidences_found.remove(&ev);
                    }
                } else if p.gg.evidences_found.contains(&ev) {
                    p.gg.evidences_found.remove(&ev);
                } else {
                    p.gg.evidences_found.insert(ev);
                    p.gg.evidences_missing.remove(&ev);
                }
            }
            if let Some((gh, discard)) = clicked_ghost_type {
                if discard {
                    if p.gg.ghosts_discarded.contains(&gh) {
                        p.gg.ghosts_discarded.remove(&gh);
                    } else {
                        p.gg.ghosts_discarded.insert(gh);
                        if p.gg.ghost_type == Some(gh) {
                            p.gg.ghost_type = None;
                        }
                    }
                } else if p.gg.ghost_type == Some(gh) {
                    p.gg.ghost_type = None;
                } else {
                    p.gg.ghost_type = Some(gh);
                }
            }
        }
        NetMode::Join { .. } => {
            if let Some(player_id) = p.local_id.0 {
                if let Some((evidence, discard)) = clicked_evidence_type {
                    p.ev_net.write(SendNetworkMessage(
                        NetworkMessage::RequestJournalEvidenceToggle(
                            RequestJournalEvidenceToggleMsg {
                                player_id,
                                evidence,
                                discard,
                            },
                        ),
                    ));
                }
                if let Some((ghost_type, discard)) = clicked_ghost_type {
                    p.ev_net.write(SendNetworkMessage(
                        NetworkMessage::RequestJournalGhostToggle(RequestJournalGhostToggleMsg {
                            player_id,
                            ghost_type,
                            discard,
                        }),
                    ));
                }
            }
        }
    }

    // --- 3. CALCULATE DERIVED STATE ---
    let possible_ghosts: Vec<GhostType> = p
        .difficulty
        .0
        .ghost_set
        .as_vec()
        .into_iter()
        .filter(|ghost_type| {
            let ghost_ev = ghost_type.evidences();
            let is_discarded = p.gg.ghosts_discarded.contains(ghost_type);

            !is_discarded
                && ghost_ev.is_superset(&p.gg.evidences_found)
                && ghost_ev.is_disjoint(&p.gg.evidences_missing)
        })
        .collect();

    // Host-exclusive: auto-select/deselect logic
    if !matches!(p.cli.net_mode, NetMode::Join { .. }) {
        // a) Auto-deselect if the currently selected ghost becomes invalid
        if let Some(selected_ghost) = p.gg.ghost_type
            && !possible_ghosts.contains(&selected_ghost)
        {
            p.gg.ghost_type = None;
        }

        // b) Auto-select if only one ghost is possible and nothing is selected
        if possible_ghosts.len() == 1 && p.gg.ghost_type.is_none() {
            p.gg.ghost_type = Some(possible_ghosts[0]);
        }
    }

    // --- 4. UPDATE UI FROM STATE ---
    for (interaction_ref, mut bgcolor, mut border_color, children, mut tui_button) in
        &mut p.interaction_query
    {
        let interaction = *interaction_ref;

        match tui_button.class {
            TruckButtonType::Ghost(gh) => {
                if p.gg.ghosts_discarded.contains(&gh) {
                    tui_button.status = TruckButtonState::Discard;
                    tui_button.disabled = false;
                } else {
                    tui_button.disabled = !possible_ghosts.contains(&gh);
                    if p.gg.ghost_type == Some(gh) {
                        tui_button.status = TruckButtonState::Pressed;
                    } else {
                        tui_button.status = TruckButtonState::Off;
                    }
                }
            }
            TruckButtonType::Evidence(ev) => {
                // Secondary check: is the gear for this evidence even available?
                let gear_available = p
                    .difficulty
                    .0
                    .truck_gear
                    .iter()
                    .filter_map(|gear_kind| Evidence::try_from(gear_kind).ok())
                    .any(|e| e == ev);

                if !gear_available {
                    tui_button.disabled = true;
                } else if p.gg.evidences_found.contains(&ev) {
                    tui_button.status = TruckButtonState::Pressed;
                    tui_button.disabled = false;
                } else if p.gg.evidences_missing.contains(&ev) {
                    tui_button.status = TruckButtonState::Discard;
                    tui_button.disabled = false;
                } else {
                    tui_button.status = TruckButtonState::Off;
                    let cannot_be_found = !possible_ghosts.is_empty()
                        && possible_ghosts.iter().all(|g| !g.evidences().contains(&ev));
                    tui_button.disabled = cannot_be_found;
                }
            }
            TruckButtonType::CraftRepellent => {
                tui_button.disabled = p.gg.ghost_type.is_none();
                // Ensure status is Off unless it was somehow changed (not by this system)
            }
            _ => {}
        }

        let current_interaction = if tui_button.disabled {
            Interaction::None
        } else {
            interaction
        };

        let mut textcolor = p.q_textcolor.get_mut(children[0]).unwrap();

        let current_border_color = tui_button.border_color(current_interaction);
        let current_background_color = tui_button.background_color(current_interaction);
        let current_text_color = tui_button.text_color(current_interaction);

        if !tui_button.blinking_hint_active {
            *border_color = BorderColor::all(current_border_color);
        }
        *bgcolor = current_background_color.into();
        textcolor.0 = current_text_color;
    }

    // Acknowledge hints (Evidence buttons)
    for (_, _, _, _, mut tui_button) in &mut p.interaction_query {
        if let TruckButtonType::Evidence(ev) = tui_button.class
            && p.gg.evidences_found.contains(&ev)
        {
            tui_button.blinking_hint_active = false;

            if let Some((hinted_evidence, _)) = p.walkie_play.evidence_hinted_not_logged_via_walkie
                && hinted_evidence == ev
            {
                const JOURNAL_HINT_THRESHOLD: u32 = 3;
                let ack_count = p
                    .profile_data
                    .times_evidence_acknowledged_in_journal
                    .entry(ev)
                    .or_insert(0);
                if *ack_count < JOURNAL_HINT_THRESHOLD {
                    *ack_count += 3; // Optimized: set it to threshold immediately
                    p.profile_data.set_changed();
                }
                p.walkie_play.clear_evidence_hint();
            }

            if let Some(potential_data) = &p.potential_id_timer.data
                && potential_data.evidence == ev
            {
                p.potential_id_timer.data = None;
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

/// System that handles network messages for the journal on the host.
fn host_handle_journal_messages_system(
    cli: Res<CliOptions>,
    mut ev_reader: MessageReader<NetworkDataEvent>,
    mut gg: ResMut<GhostGuess>,
) {
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        return;
    }

    for ev in ev_reader.read() {
        match &ev.message {
            NetworkMessage::JournalUpdate(msg) => {
                let JournalUpdateMsg {
                    player_id,
                    ghost_type,
                    evidences_found,
                    evidences_missing,
                } = msg;
                debug!(
                    "Journal: Received JournalUpdate from client {:?}",
                    player_id
                );
                gg.ghost_type = *ghost_type;
                gg.evidences_found = evidences_found.iter().cloned().collect();
                gg.evidences_missing = evidences_missing.iter().cloned().collect();
            }
            NetworkMessage::RequestJournalEvidenceToggle(msg) => {
                let RequestJournalEvidenceToggleMsg {
                    player_id,
                    evidence,
                    discard,
                } = msg;
                debug!(
                    "Journal: Received RequestJournalEvidenceToggle from client {:?} for {:?} (discard: {})",
                    player_id, evidence, discard
                );
                if *discard {
                    if gg.evidences_missing.contains(evidence) {
                        gg.evidences_missing.remove(evidence);
                    } else {
                        gg.evidences_missing.insert(*evidence);
                        gg.evidences_found.remove(evidence);
                    }
                } else if gg.evidences_found.contains(evidence) {
                    gg.evidences_found.remove(evidence);
                } else if gg.evidences_missing.contains(evidence) {
                    gg.evidences_missing.remove(evidence);
                } else {
                    gg.evidences_found.insert(*evidence);
                }
            }
            NetworkMessage::RequestJournalGhostToggle(msg) => {
                let RequestJournalGhostToggleMsg {
                    player_id,
                    ghost_type,
                    discard,
                } = msg;
                debug!(
                    "Journal: Received RequestJournalGhostToggle from client {:?} for {:?} (discard: {})",
                    player_id, ghost_type, discard
                );
                if *discard {
                    if gg.ghosts_discarded.contains(ghost_type) {
                        gg.ghosts_discarded.remove(ghost_type);
                    } else {
                        gg.ghosts_discarded.insert(*ghost_type);
                        if gg.ghost_type == Some(*ghost_type) {
                            gg.ghost_type = None;
                        }
                    }
                } else if gg.ghost_type == Some(*ghost_type) {
                    gg.ghost_type = None;
                } else {
                    gg.ghost_type = Some(*ghost_type);
                    gg.ghosts_discarded.remove(ghost_type);
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_message::<ForceDiscardEvidenceEvent>()
        .add_systems(Update, ghost_guess_system)
        .add_systems(
            Update,
            (
                force_discard_evidence_system,
                host_handle_journal_messages_system,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(Update, button_system.run_if(in_state(AppState::InGame)));
}
