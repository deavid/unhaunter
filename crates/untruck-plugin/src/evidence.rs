use super::uibutton::{TruckButtonState, TruckButtonType, TruckUIButton};
use crate::types::evidence_status::EvidenceStatus;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use ungear_core::components::core::EvidenceSensor;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::looking_gear::LookingGear;
use unnet_core::messages::{NetworkMessage, RequestJournalEvidenceToggleMsg, SendNetworkMessage};
use unnet_core::resources::LocalPlayer;
use unplayer_core::components::{MainPlayer, PlayerInputMapping, PlayerSprite};
use unprofile_core::profile::PlayerProfileData;
use untypes_core::cli::{CliOptions, NetMode};
use untypes_core::states::AppState;
use unui_core::components::game_ui::EvidenceUI;

pub(crate) fn update_evidence_ui(
    q_gear: Query<(&PlayerSprite, &PlayerGear), With<MainPlayer>>,
    q_sensor: Query<&EvidenceSensor>,
    mut qs: Query<Entity, With<EvidenceUI>>,
    interaction_query: Query<&TruckUIButton, With<Button>>,
    mut writer: TextUiWriter,
    looking_gear: Res<LookingGear>,
) {
    for (_ps, playergear) in q_gear.iter() {
        for txt_entity in qs.iter_mut() {
            let hand_entity = match looking_gear.hand() {
                unfoundation_core::types::gear::Hand::Left => playergear.left_hand,
                unfoundation_core::types::gear::Hand::Right => playergear.right_hand,
            };
            let o_evidence = hand_entity
                .and_then(|e| q_sensor.get(e).ok())
                .map(|s| s.evidence);

            let ev_state = match o_evidence {
                Some(ev) => interaction_query
                    .iter()
                    .find(|t| t.class == TruckButtonType::Evidence(ev))
                    .map(|t| t.status)
                    .unwrap_or(TruckButtonState::Off),
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
    mut interaction_query: Query<&mut TruckUIButton, With<Button>>,
    looking_gear: Res<LookingGear>,
    mut profile_data: ResMut<Persistent<PlayerProfileData>>,
    cli: Res<CliOptions>,
    local_id: Res<LocalPlayer>,
    mut ev_net: MessageWriter<SendNetworkMessage>,
) {
    for (input_mapping, playergear) in &players {
        let hand_entity = match looking_gear.hand() {
            unfoundation_core::types::gear::Hand::Left => playergear.left_hand,
            unfoundation_core::types::gear::Hand::Right => playergear.right_hand,
        };
        let Some(evidence) = hand_entity
            .and_then(|e| q_sensor.get(e).ok())
            .map(|s| s.evidence)
        else {
            continue;
        };

        if keyboard_input.just_pressed(input_mapping.controls.change_evidence) {
            match cli.net_mode {
                NetMode::Offline | NetMode::Host { .. } => {
                    for mut t in &mut interaction_query {
                        if t.class == TruckButtonType::Evidence(evidence) {
                            // Call pressed() first to change the button state
                            t.pressed();

                            // Track gear acknowledgment if button is now pressed (evidence found)
                            if t.status == TruckButtonState::Pressed {
                                const GEAR_HINT_THRESHOLD: u32 = 3; // Same threshold as journal
                                let ack_count_entry = profile_data
                                    .times_evidence_acknowledged_on_gear
                                    .entry(evidence)
                                    .or_insert(0);

                                if *ack_count_entry < GEAR_HINT_THRESHOLD {
                                    *ack_count_entry += 1;
                                    profile_data.set_changed(); // Mark Persistent data as changed
                                    // info!("Gear hint for {:?} acknowledged. New count: {}", evidence, *ack_count_entry);
                                }
                            }
                        }
                    }
                }
                NetMode::Join { .. } => {
                    if let Some(player_id) = local_id.0 {
                        ev_net.write(SendNetworkMessage(
                            NetworkMessage::RequestJournalEvidenceToggle(
                                RequestJournalEvidenceToggleMsg {
                                    player_id,
                                    evidence,
                                    discard: false,
                                },
                            ),
                        ));
                    }
                }
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
