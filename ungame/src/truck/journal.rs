use super::{
    uibutton::{TruckButtonState, TruckButtonType, TruckUIButton},
    GhostGuess, TruckUIEvent, TruckUIGhostGuess,
};
use crate::game::GameConfig;
use crate::player::PlayerSprite;
use bevy::{prelude::*, utils::HashSet};
use uncore::types::evidence::Evidence;
use ungear::components::playergear::PlayerGear;

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
) {
    let mut selected_evidences_found = HashSet::<Evidence>::new();
    let mut selected_evidences_missing = HashSet::<Evidence>::new();
    let mut evidences_possible = HashSet::<Evidence>::new();
    let mut evidences_all_ghosts = HashSet::<Evidence>::new();
    let mut first_ghost = true;
    let mut new_ghost_selected = None;
    for (interaction, _color, _border_color, _children, mut tui_button) in &mut interaction_query {
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
                ev_truckui.send(truckui_event);
            }
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
        border_color.0 = tui_button.border_color(interaction);
        *color = tui_button.background_color(interaction).into();
        textcolor.0 = tui_button.text_color(interaction);
    }
    gg.ghost_type = ghost_selected;
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
