use crate::evidence_status::EvidenceStatus;
use bevy::picking::events::{Click, Pointer};
use bevy::prelude::*;
use uncommon_states_core::UIContextState;
use ungear_core::components::core::EvidenceSensor;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::looking_gear::LookingGear;
use ungear_core::types::gear::equipment::Hand;
use ungear_core::ui::{EvidenceClickTarget, EvidenceUI};
use uninput_core::components::PlayerInputMapping;
use uninvestigation_core::evidence::Evidence;
use uninvestigation_core::messages::RequestJournalEvidenceToggle;
use uninvestigation_core::resources::ghost_guess::GhostGuess;
use unplayer_core::components::{MainPlayer, PlayerSprite};
use untruck_core::types::truck_button::TruckButtonState;

fn toggle_evidence(
    evidence: Evidence,
    ev_evidence_toggle: &mut MessageWriter<RequestJournalEvidenceToggle>,
    gg: &GhostGuess,
) {
    let mark_as_found = !gg.evidences_found.contains(&evidence);
    ev_evidence_toggle.write(RequestJournalEvidenceToggle {
        evidence,
        discard: false,
        mark_as_found,
    });
}

pub(crate) fn update_evidence_ui(
    q_gear: Query<(&PlayerSprite, &PlayerGear), With<MainPlayer>>,
    q_sensor: Query<&EvidenceSensor>,
    mut qs: Query<Entity, With<EvidenceUI>>,
    gg: Res<GhostGuess>,
    mut writer: TextUiWriter,
    looking_gear: Res<LookingGear>,
) {
    for (_ps, playergear) in q_gear.iter() {
        for txt_entity in qs.iter_mut() {
            let hand_entity = match looking_gear.hand() {
                Hand::Left => playergear.left_hand,
                Hand::Right => playergear.right_hand,
            };
            let o_evidence = hand_entity
                .and_then(|e| q_sensor.get(e).ok())
                .map(|s| s.evidence);

            let ev_state = match o_evidence {
                Some(ev) => {
                    if gg.evidences_found.contains(&ev) {
                        TruckButtonState::Pressed
                    } else if gg.evidences_missing.contains(&ev) {
                        TruckButtonState::Discard
                    } else {
                        TruckButtonState::Off
                    }
                }
                None => TruckButtonState::Off,
            };
            let status = EvidenceStatus::from_gearkind(o_evidence, ev_state);
            if let Some((_entity, _depth, mut text, _font, _color, _)) = writer.get(txt_entity, 1)
                && *text != status.title
            {
                *text = status.title;
            }
            if let Some((_entity, _depth, mut text, _font, mut color, _)) =
                writer.get(txt_entity, 2)
                && *text != status.status_game
            {
                *text = status.status_game;
                *color = TextColor(status.status_color);
            }
            if let Some((_entity, _depth, mut text, _font, _color, _)) = writer.get(txt_entity, 3)
                && *text != status.help_text
            {
                *text = status.help_text;
            }
        }
    }
}

pub(crate) fn keyboard_evidence(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    players: Query<(&PlayerInputMapping, &PlayerGear), With<MainPlayer>>,
    q_sensor: Query<&EvidenceSensor>,
    looking_gear: Res<LookingGear>,
    mut ev_evidence_toggle: MessageWriter<RequestJournalEvidenceToggle>,
    gg: Res<GhostGuess>,
) {
    for (input_mapping, playergear) in &players {
        let hand_entity = match looking_gear.hand() {
            Hand::Left => playergear.left_hand,
            Hand::Right => playergear.right_hand,
        };
        let Some(evidence) = hand_entity
            .and_then(|e| q_sensor.get(e).ok())
            .map(|s| s.evidence)
        else {
            continue;
        };

        if keyboard_input.just_pressed(input_mapping.controls.change_evidence) {
            toggle_evidence(evidence, &mut ev_evidence_toggle, &gg);
        }
    }
}

pub(crate) fn click_evidence(
    mut click_events: MessageReader<Pointer<Click>>,
    players: Query<&PlayerGear, With<MainPlayer>>,
    q_sensor: Query<&EvidenceSensor>,
    q_evidence_ui: Query<(), With<EvidenceUI>>,
    q_evidence_click: Query<(), With<EvidenceClickTarget>>,
    q_parent: Query<&ChildOf>,
    looking_gear: Res<LookingGear>,
    mut ev_evidence_toggle: MessageWriter<RequestJournalEvidenceToggle>,
    gg: Res<GhostGuess>,
) {
    let Ok(playergear) = players.single() else {
        return;
    };

    let hand_entity = match looking_gear.hand() {
        Hand::Left => playergear.left_hand,
        Hand::Right => playergear.right_hand,
    };
    let Some(evidence) = hand_entity
        .and_then(|e| q_sensor.get(e).ok())
        .map(|s| s.evidence)
    else {
        return;
    };

    for click_event in click_events.read() {
        if click_event.button != PointerButton::Primary {
            continue;
        }

        let mut hit = click_event.entity;
        let mut is_target = false;

        for _ in 0..5 {
            // Max depth traversal
            if q_evidence_ui.contains(hit) || q_evidence_click.contains(hit) {
                is_target = true;
                break;
            }
            if let Ok(parent) = q_parent.get(hit) {
                hit = parent.parent();
            } else {
                break;
            }
        }

        if is_target {
            toggle_evidence(evidence, &mut ev_evidence_toggle, &gg);
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        update_evidence_ui.run_if(in_state(UIContextState::InGame)),
    )
    .add_systems(
        Update,
        (keyboard_evidence, click_evidence).run_if(in_state(UIContextState::InGame)),
    );
}
