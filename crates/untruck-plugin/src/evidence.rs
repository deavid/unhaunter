use super::uibutton::TruckButtonState;
use crate::types::evidence_status::EvidenceStatus;
use bevy::prelude::*;
use ungear_core::components::core::EvidenceSensor;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::looking_gear::LookingGear;
use ungear_core::types::gear::equipment::Hand;
use unghost_core::resources::ghost_guess::GhostGuess;
use unplayer_core::components::{MainPlayer, PlayerInputMapping, PlayerSprite};
use unreplicon_core::messages::RequestJournalEvidenceToggle;
use untypes_core::roles::AuthorityRole;
use untypes_core::states::AppState;
use unui_core::components::game_ui::EvidenceUI;

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
    authority: Option<Res<AuthorityRole>>,
    mut ev_evidence_toggle: MessageWriter<RequestJournalEvidenceToggle>,
    mut gg: ResMut<GhostGuess>,
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
            if authority.is_some() {
                if gg.evidences_found.contains(&evidence) {
                    gg.evidences_found.remove(&evidence);
                } else {
                    // If it was missing/discarded, we reset it to found.
                    gg.evidences_missing.remove(&evidence);
                    gg.evidences_found.insert(evidence);
                }
            } else {
                let mark_as_found = !gg.evidences_found.contains(&evidence);
                ev_evidence_toggle.write(RequestJournalEvidenceToggle {
                    evidence,
                    discard: false,
                    mark_as_found,
                });
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        update_evidence_ui.run_if(in_state(AppState::InGame)),
    )
    .add_systems(Update, keyboard_evidence.run_if(in_state(AppState::InGame)));
}
