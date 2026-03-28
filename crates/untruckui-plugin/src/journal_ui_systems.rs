use crate::colors;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use ungear_core::difficulty_ext::DifficultyGearExt;
use unghost_core::difficulty_ext::DifficultyGhostExt;
use uninvestigation_core::evidence::Evidence;
use uninvestigation_core::ghost::GhostType;
use uninvestigation_core::resources::ghost_guess::GhostGuess;
use uninvestigation_core::resources::potential_id_timer::PotentialIDTimer;
use unprofile_core::profile::PlayerProfileData;
use unreplicon_core::messages::{RequestJournalEvidenceToggle, RequestJournalGhostToggle};
use unreplicon_core::resources::AuthorityRole;
use untruck_core::components::truck_ui_button::TruckUIButton;
use untruck_core::events::truck::TruckUIEvent;
use untruck_core::types::truck_button::{TruckButtonState, TruckButtonType};
use unwalkie_core::resources::WalkiePlay;

#[derive(SystemParam)]
pub(crate) struct JournalButtonParams<'w, 's> {
    pub interaction_query: Query<
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
    pub q_textcolor: Query<'w, 's, &'static mut TextColor>,
    pub gg: ResMut<'w, GhostGuess>,
    pub ev_truckui: MessageWriter<'w, TruckUIEvent>,
    pub walkie_play: ResMut<'w, WalkiePlay>,
    pub profile_data: ResMut<'w, Persistent<PlayerProfileData>>,
    pub potential_id_timer: ResMut<'w, PotentialIDTimer>,
    pub keyboard_input: Res<'w, ButtonInput<KeyCode>>,
    pub difficulty: Res<'w, CurrentDifficulty>,
    pub authority: Option<Res<'w, AuthorityRole>>,
    pub ev_evidence_toggle: MessageWriter<'w, RequestJournalEvidenceToggle>,
    pub ev_ghost_toggle: MessageWriter<'w, RequestJournalGhostToggle>,
}

pub(crate) fn button_system(mut p: JournalButtonParams) {
    let mut clicked_ghost_type: Option<(GhostType, bool)> = None;
    let mut clicked_evidence_type: Option<(Evidence, bool)> = None;

    let shift_pressed = p.keyboard_input.pressed(KeyCode::ShiftLeft)
        || p.keyboard_input.pressed(KeyCode::ShiftRight);

    // --- 1. GATHER INPUTS ---
    for (interaction, _, _, _, tui_button) in &mut p.interaction_query {
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
                    // For CraftRepellent, ExitTruck, EndMission: emit the event directly.
                    let truckui_event = match tui_button.class {
                        TruckButtonType::CraftRepellent => Some(TruckUIEvent::CraftRepellent),
                        TruckButtonType::ExitTruck => Some(TruckUIEvent::ExitTruck),
                        TruckButtonType::EndMission => Some(TruckUIEvent::EndMission),
                        _ => None,
                    };
                    if let Some(ev) = truckui_event {
                        p.ev_truckui.write(ev);
                    }
                }
            }
        }
    }

    // --- 2. UPDATE STATE (HOST) OR SEND MESSAGES (CLIENT) ---
    if p.authority.is_some() {
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
    } else {
        if let Some((evidence, discard)) = clicked_evidence_type {
            let mark_as_found = !discard && !p.gg.evidences_found.contains(&evidence);
            p.ev_evidence_toggle.write(RequestJournalEvidenceToggle {
                evidence,
                discard,
                mark_as_found,
            });
        }
        if let Some((ghost_type, discard)) = clicked_ghost_type {
            let new_guess = if discard {
                Some(ghost_type)
            } else if p.gg.ghost_type == Some(ghost_type) {
                None
            } else {
                Some(ghost_type)
            };
            p.ev_ghost_toggle.write(RequestJournalGhostToggle {
                discard,
                ghost_type: new_guess,
            });
        }
    }

    // --- 3. CALCULATE DERIVED STATE ---
    let possible_ghosts: Vec<GhostType> = p
        .difficulty
        .0
        .ghost_set()
        .as_vec()
        .into_iter()
        .filter(|ghost_type: &GhostType| {
            let ghost_ev = ghost_type.evidences();
            let is_discarded = p.gg.ghosts_discarded.contains(ghost_type);

            !is_discarded
                && ghost_ev.is_superset(&p.gg.evidences_found)
                && ghost_ev.is_disjoint(&p.gg.evidences_missing)
        })
        .collect();

    // Host-exclusive: auto-select/deselect logic
    if p.authority.is_some() {
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
                    .truck_gear()
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
                        && possible_ghosts
                            .iter()
                            .all(|g: &GhostType| !g.evidences().contains(&ev));
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

        let current_border_color = {
            let color = match tui_button.class {
                TruckButtonType::Evidence(_) => match current_interaction {
                    Interaction::Pressed => colors::TRUCKUI_ACCENT3_COLOR,
                    Interaction::Hovered => colors::TRUCKUI_TEXT_COLOR,
                    Interaction::None => colors::TRUCKUI_ACCENT2_COLOR,
                },
                TruckButtonType::Ghost(_) => match current_interaction {
                    Interaction::Pressed => colors::TRUCKUI_ACCENT3_COLOR,
                    Interaction::Hovered => colors::TRUCKUI_ACCENT_COLOR,
                    Interaction::None => Color::NONE,
                },
                TruckButtonType::ExitTruck | TruckButtonType::CraftRepellent => {
                    match current_interaction {
                        Interaction::Pressed => colors::BUTTON_EXIT_TRUCK_TXTCOLOR,
                        Interaction::Hovered => colors::BUTTON_EXIT_TRUCK_TXTCOLOR,
                        Interaction::None => colors::BUTTON_EXIT_TRUCK_FGCOLOR,
                    }
                }
                TruckButtonType::EndMission => match current_interaction {
                    Interaction::Pressed => colors::BUTTON_END_MISSION_TXTCOLOR,
                    Interaction::Hovered => colors::BUTTON_END_MISSION_TXTCOLOR,
                    Interaction::None => colors::BUTTON_END_MISSION_FGCOLOR,
                },
            };
            let alpha_disabled = if tui_button.disabled { 0.05 } else { 1.0 };
            BorderColor::all(color.with_alpha(color.alpha() * alpha_disabled))
        };
        if !tui_button.blinking_hint_active {
            *border_color = current_border_color;
        }

        let current_bg_color = {
            let color = match tui_button.class {
                TruckButtonType::Evidence(_) => match tui_button.status {
                    TruckButtonState::Off => colors::TRUCKUI_BGCOLOR,
                    TruckButtonState::Pressed => Color::srgb(0.2, 0.8, 0.3),
                    TruckButtonState::Discard => colors::BUTTON_END_MISSION_FGCOLOR,
                },
                TruckButtonType::Ghost(_) => match tui_button.status {
                    TruckButtonState::Off => colors::TRUCKUI_BGCOLOR,
                    TruckButtonState::Pressed => Color::srgb(0.2, 0.8, 0.3),
                    TruckButtonState::Discard => colors::BUTTON_END_MISSION_FGCOLOR,
                },
                TruckButtonType::ExitTruck | TruckButtonType::CraftRepellent => {
                    match current_interaction {
                        Interaction::Pressed => colors::BUTTON_EXIT_TRUCK_FGCOLOR,
                        Interaction::Hovered => colors::BUTTON_EXIT_TRUCK_BGCOLOR,
                        Interaction::None => colors::BUTTON_EXIT_TRUCK_BGCOLOR,
                    }
                }
                TruckButtonType::EndMission => match current_interaction {
                    Interaction::Pressed => colors::BUTTON_END_MISSION_FGCOLOR,
                    Interaction::Hovered => colors::BUTTON_END_MISSION_BGCOLOR,
                    Interaction::None => colors::BUTTON_END_MISSION_BGCOLOR,
                },
            };
            let color = if tui_button.disabled {
                let color = color.with_alpha(color.alpha() * 0.5);
                color.with_luminance(color.luminance() * 0.5)
            } else {
                color
            };
            BackgroundColor(color)
        };
        *bgcolor = current_bg_color;

        let current_text_color = {
            let color = match tui_button.class {
                TruckButtonType::Evidence(_) => match tui_button.status {
                    TruckButtonState::Off => colors::TRUCKUI_TEXT_COLOR,
                    TruckButtonState::Pressed => Color::BLACK,
                    TruckButtonState::Discard => colors::TRUCKUI_TEXT_COLOR.with_alpha(0.5),
                },
                TruckButtonType::Ghost(_) => match tui_button.status {
                    TruckButtonState::Off => colors::TRUCKUI_TEXT_COLOR,
                    TruckButtonState::Pressed => Color::BLACK,
                    TruckButtonState::Discard => colors::TRUCKUI_TEXT_COLOR.with_alpha(0.5),
                },
                TruckButtonType::ExitTruck | TruckButtonType::CraftRepellent => {
                    colors::BUTTON_EXIT_TRUCK_TXTCOLOR
                }
                TruckButtonType::EndMission => colors::BUTTON_END_MISSION_TXTCOLOR,
            };
            let alpha_disabled = if tui_button.disabled { 0.1 } else { 1.0 };
            TextColor(color.with_alpha(color.alpha() * alpha_disabled))
        };
        textcolor.0 = current_text_color.0;
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
