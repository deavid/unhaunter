use super::uibutton::{TruckButtonState, TruckButtonType, TruckUIButton};
use crate::components::truck::TruckUIGhostGuess;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use bevy_platform::collections::HashSet;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unevents_core::events::truck::TruckUIEvent;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use ungear_core::types::gear::kind::GearKind;
use ungearitems_core::components::repellentflask::RepellentFlask;
use unghost_core::resources::ghost_guess::GhostGuess;
use unghost_core::resources::potential_id_timer::PotentialIDTimer;
use unghost_core::types::evidence::Evidence;
use unghost_core::types::ghost::types::GhostType;
use unnet_core::messages::{NetworkDataEvent, NetworkMessage};
use unnet_core::network_id::NetworkId;
use unnet_core::resources::LocalPlayer;
use unplayer_core::components::{MainPlayer, PlayerSprite};
use unprofile_core::profile::PlayerProfileData;
use untruck_core::journal::ForceDiscardEvidenceEvent;
use untypes_core::cli::{CliOptions, NetMode};
use untypes_core::states::{AppState, GameState};
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
    q_gear: Query<'w, 's, (&'static PlayerSprite, &'static mut PlayerGear), With<MainPlayer>>,
    commands: Commands<'w, 's>,
    gear_registry: Res<'w, GearSpawnerRegistry>,
    q_repellent: Query<'w, 's, &'static mut RepellentFlask>,
    q_gearkind: Query<'w, 's, &'static GearKind>,
    cli: Res<'w, CliOptions>,
    ev_net: MessageWriter<'w, NetworkDataEvent>,
    local_id: Res<'w, LocalPlayer>,
}

fn button_system(mut p: JournalButtonParams) {
    let mut selected_evidences_found = HashSet::<Evidence>::new();
    let mut selected_evidences_missing = HashSet::<Evidence>::new();
    let mut clicked_ghost_type: Option<GhostType> = None;

    let shift_pressed = p.keyboard_input.pressed(KeyCode::ShiftLeft)
        || p.keyboard_input.pressed(KeyCode::ShiftRight);

    // --- 1. GATHER INPUTS ---
    // First pass: Handle evidence button clicks and detect ghost button clicks.
    for (interaction, _, _, _, mut tui_button) in &mut p.interaction_query {
        // Skip buttons that use hold timer or are currently disabled from a previous frame
        if tui_button.disabled || tui_button.hold_duration.is_some() || tui_button.computer_locked {
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
                        if p.gg.ghost_type == Some(ghost_type) {
                            p.gg.ghost_type = None;
                        }
                    } else {
                        tui_button.pressed();
                        clicked_ghost_type = Some(ghost_type);
                    }
                }
                TruckButtonType::CraftRepellent => {
                    if let Some(ghost_type) = p.gg.ghost_type {
                        match p.cli.net_mode {
                            NetMode::Offline | NetMode::Host { .. } => {
                                for (_player, mut gear) in p.q_gear.iter_mut() {
                                    crate::craft_repellent::craft_repellent(
                                        &mut p.commands,
                                        &p.gear_registry,
                                        &mut gear,
                                        ghost_type,
                                        &mut p.q_repellent,
                                        &p.q_gearkind,
                                    );
                                }
                            }
                            NetMode::Join { .. } => {
                                if let Some(player_id) = p.local_id.0 {
                                    p.ev_net.write(NetworkDataEvent {
                                        message: NetworkMessage::CraftRepellent {
                                            player_id,
                                            ghost_type,
                                        },
                                    });
                                }
                            }
                        }
                    }
                    if let Some(truckui_event) = tui_button.pressed() {
                        p.ev_truckui.write(truckui_event);
                    }
                }
                _ => {
                    // For Craft, End, etc.
                    if let Some(truckui_event) = tui_button.pressed() {
                        p.ev_truckui.write(truckui_event);
                    }
                }
            }
        }
    }

    // After handling clicks, now collect the final state of all evidence buttons
    for (_, _, _, _, tui_button) in &p.interaction_query {
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

    // Check if evidence states have changed
    let evidence_states_changed = p.gg.evidences_found != selected_evidences_found
        || p.gg.evidences_missing != selected_evidences_missing;

    // Only log evidence states if there are changes
    if evidence_states_changed {
        debug!(
            "Journal: Evidence found: {:?}, Evidence missing: {:?}",
            selected_evidences_found, selected_evidences_missing
        );
    }

    // Update the GhostGuess if there are changes
    if evidence_states_changed {
        debug!(
            "Journal: Updating evidences_found: {:?} -> {:?}",
            p.gg.evidences_found, selected_evidences_found
        );
        debug!(
            "Journal: Updating evidences_missing: {:?} -> {:?}",
            p.gg.evidences_missing, selected_evidences_missing
        );
        p.gg.evidences_found = selected_evidences_found.clone();
        p.gg.evidences_missing = selected_evidences_missing.clone();
    }

    let possible_ghosts: Vec<GhostType> = p
        .difficulty
        .0
        .ghost_set
        .as_vec()
        .into_iter()
        .filter(|ghost_type| {
            let ghost_ev = ghost_type.evidences();
            let mut is_discarded = false;
            for (_, _, _, _, tui_button) in &p.interaction_query {
                if let TruckButtonType::Ghost(gh) = tui_button.class
                    && gh == *ghost_type
                    && tui_button.status == TruckButtonState::Discard
                {
                    is_discarded = true;
                    break;
                }
            }
            !is_discarded
                && ghost_ev.is_superset(&selected_evidences_found)
                && ghost_ev.is_disjoint(&selected_evidences_missing)
        })
        .collect();

    // a) Handle manual click on a ghost button
    if let Some(clicked_ghost) = clicked_ghost_type {
        if p.gg.ghost_type == Some(clicked_ghost) {
            p.gg.ghost_type = None;
        } else {
            p.gg.ghost_type = Some(clicked_ghost);
        }
    }

    // b) Auto-deselect if the currently selected ghost becomes invalid
    if let Some(selected_ghost) = p.gg.ghost_type
        && !possible_ghosts.contains(&selected_ghost)
    {
        p.gg.ghost_type = None;
    }

    // c) Auto-select if only one ghost is possible and nothing is selected
    if possible_ghosts.len() == 1 && p.gg.ghost_type.is_none() {
        p.gg.ghost_type = Some(possible_ghosts[0]);
    }

    // --- d) SYNC GHOSTGUESS OVER NETWORK ---
    if p.gg.is_changed()
        && matches!(p.cli.net_mode, NetMode::Join { .. })
        && let Some(player_id) = p.local_id.0
    {
        p.ev_net.write(NetworkDataEvent {
            message: NetworkMessage::JournalUpdate {
                player_id,
                ghost_type: p.gg.ghost_type,
                evidences_found: p.gg.evidences_found.iter().cloned().collect(),
                evidences_missing: p.gg.evidences_missing.iter().cloned().collect(),
            },
        });
    }

    // --- 3. UPDATE UI FROM STATE ---
    // Second pass: Update visuals and disabled states of all buttons based on the now-finalized GhostGuess.
    for (interaction_ref, mut bgcolor, mut border_color, children, mut tui_button) in
        &mut p.interaction_query
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
            if p.gg.ghost_type == Some(gh) && tui_button.status != TruckButtonState::Discard {
                tui_button.status = TruckButtonState::Pressed;
            } else if tui_button.status != TruckButtonState::Discard {
                tui_button.status = TruckButtonState::Off;
            }
        }

        // --- NEW LOGIC FOR EVIDENCE BUTTONS ---
        if let TruckButtonType::Evidence(ev) = tui_button.class {
            // Primary check: is the gear for this evidence even available?
            if !p
                .difficulty
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
            let disabled = p.gg.ghost_type.is_none();
            tui_button.disabled = disabled;
        }

        let current_interaction = if tui_button.disabled {
            Interaction::None
        } else {
            interaction
        };

        let mut textcolor = p.q_textcolor.get_mut(children[0]).unwrap();

        // Default color calculation
        let current_border_color = tui_button.border_color(current_interaction);
        let current_background_color = tui_button.background_color(current_interaction);
        let current_text_color = tui_button.text_color(current_interaction);

        if !tui_button.blinking_hint_active {
            *border_color = BorderColor::all(current_border_color);
        }
        *bgcolor = current_background_color.into();
        textcolor.0 = current_text_color;
    }

    // Update GhostGuess resource with the latest evidence sets (only if changed)

    let final_found_changed = p.gg.evidences_found != selected_evidences_found;
    let final_missing_changed = p.gg.evidences_missing != selected_evidences_missing;

    if final_found_changed {
        debug!(
            "Journal: Final update evidences_found: {:?} -> {:?}",
            p.gg.evidences_found, selected_evidences_found
        );
        p.gg.evidences_found = selected_evidences_found;
    }
    if final_missing_changed {
        debug!(
            "Journal: Final update evidences_missing: {:?} -> {:?}",
            p.gg.evidences_missing, selected_evidences_missing
        );
        p.gg.evidences_missing = selected_evidences_missing;
    }

    // Acknowledge hints
    for (_interaction, _, _, _, tui_button) in &p.interaction_query {
        if let TruckButtonType::Evidence(clicked_evidence_type) = tui_button.class
            && tui_button.status == TruckButtonState::Pressed
        {
            if let Some((hinted_evidence, _)) = p.walkie_play.evidence_hinted_not_logged_via_walkie
                && hinted_evidence == clicked_evidence_type
            {
                const JOURNAL_HINT_THRESHOLD: u32 = 3;
                let ack_count = p
                    .profile_data
                    .times_evidence_acknowledged_in_journal
                    .entry(clicked_evidence_type)
                    .or_insert(0);
                if *ack_count < JOURNAL_HINT_THRESHOLD {
                    *ack_count += 1;
                    p.profile_data.set_changed();
                }
                p.walkie_play.clear_evidence_hint();
            }

            if let Some(potential_data) = &p.potential_id_timer.data
                && potential_data.evidence == clicked_evidence_type
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
    mut commands: Commands,
    gear_registry: Res<GearSpawnerRegistry>,
    mut q_players: Query<(&NetworkId, &mut PlayerGear), Without<MainPlayer>>,
    mut q_repellent: Query<&mut RepellentFlask>,
    q_gearkind: Query<&GearKind>,
) {
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        return;
    }

    for ev in ev_reader.read() {
        match &ev.message {
            NetworkMessage::JournalUpdate {
                player_id,
                ghost_type,
                evidences_found,
                evidences_missing,
            } => {
                debug!(
                    "Journal: Received JournalUpdate from client {:?}",
                    player_id
                );
                gg.ghost_type = *ghost_type;
                gg.evidences_found = evidences_found.iter().cloned().collect();
                gg.evidences_missing = evidences_missing.iter().cloned().collect();
            }
            NetworkMessage::CraftRepellent {
                player_id,
                ghost_type,
            } => {
                debug!(
                    "Journal: Received CraftRepellent from client {:?} for {:?}",
                    player_id, ghost_type
                );
                for (id, mut gear) in q_players.iter_mut() {
                    if id == player_id {
                        crate::craft_repellent::craft_repellent(
                            &mut commands,
                            &gear_registry,
                            &mut gear,
                            *ghost_type,
                            &mut q_repellent,
                            &q_gearkind,
                        );
                    }
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
        .add_systems(Update, button_system.run_if(in_state(GameState::Truck)));
}
