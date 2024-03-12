use crate::{
    colors,
    gear::playergear::PlayerGear,
    ghost_definitions::Evidence,
    player::PlayerSprite,
    root::{self, GameAssets},
    truck::{TruckButtonState, TruckButtonType, TruckUIButton},
};
use bevy::prelude::*;

use super::GameConfig;

#[derive(Component, Debug)]
pub struct EvidenceUI;

pub fn setup_ui_evidence(parent: &mut ChildBuilder, handles: &GameAssets) {
    let text_bundle = TextBundle::from_sections([
            TextSection{value: "Freezing temps:".into(), 
                style: TextStyle {
                    font: handles.fonts.chakra.w400_regular.clone(),
                    font_size: 22.0,
                    color: colors::INVENTORY_STATS_COLOR.with_a(1.0),
                },
            },
            TextSection{value: " [+] Evidence Found\n".into(), 
                style: TextStyle {
                    font: handles.fonts.victormono.w600_semibold.clone(),
                    font_size: 20.0,
                    color: Color::GREEN.with_a(0.4),
                },
            },
            TextSection{value: "The ghost and the breach will make the ambient colder.\nSome ghosts will make the temperature drop below 0.0ºC.".into(), 
                style: TextStyle {
                    font: handles.fonts.chakra.w300_light.clone(),
                    font_size: 20.0,
                    color: colors::INVENTORY_STATS_COLOR,
                },
            },
        ]);
    parent.spawn(text_bundle).insert(EvidenceUI);
}

pub fn update_evidence_ui(
    gc: Res<GameConfig>,
    q_gear: Query<(&PlayerSprite, &PlayerGear)>,
    mut qs: Query<&mut Text, With<EvidenceUI>>,
    interaction_query: Query<&TruckUIButton, With<Button>>,
) {
    for (ps, playergear) in q_gear.iter() {
        if gc.player_id == ps.id {
            for mut txt in qs.iter_mut() {
                let o_evidence = Evidence::try_from(&playergear.right_hand.kind).ok();
                let ev_state = match o_evidence {
                    Some(ev) => interaction_query
                        .iter()
                        .find(|t| t.class == TruckButtonType::Evidence(ev))
                        .map(|t| t.status)
                        .unwrap_or(TruckButtonState::Off),
                    None => TruckButtonState::Off,
                };
                let status = EvidenceStatus::from_gearkind(o_evidence, ev_state);

                if txt.sections[0].value != status.title {
                    txt.sections[0].value = status.title;
                }
                if txt.sections[1].value != status.status {
                    txt.sections[1].value = status.status;
                    txt.sections[1].style.color = status.status_color;
                }
                if txt.sections[2].value != status.help_text {
                    txt.sections[2].value = status.help_text;
                }
            }
        }
    }
}

pub fn keyboard_evidence(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gc: Res<GameConfig>,
    players: Query<(&PlayerSprite, &PlayerGear)>,
    mut interaction_query: Query<&mut TruckUIButton, With<Button>>,
) {
    for (player, playergear) in &players {
        if gc.player_id != player.id {
            continue;
        }
        let Ok(evidence) = Evidence::try_from(&playergear.right_hand.kind) else {
            continue;
        };
        if keyboard_input.just_pressed(player.controls.change_evidence) {
            for mut t in &mut interaction_query {
                if t.class == TruckButtonType::Evidence(evidence) {
                    t.pressed();
                }
            }
        }
    }
}

pub struct EvidenceStatus {
    pub title: String,
    pub status: String,
    pub status_color: Color,
    pub help_text: String,
}

impl EvidenceStatus {
    pub fn from_gearkind(o_evidence: Option<Evidence>, ev_state: TruckButtonState) -> Self {
        let Some(evidence) = o_evidence else {
            return Self {
                title: "".into(),
                status: "".into(),
                status_color: colors::INVENTORY_STATS_COLOR,
                help_text: "No evidence for selected gear.".into(),
            };
        };

        let title: String = format!("{}: ", evidence.name());

        let help_text: String = evidence.help_text().into();

        let status: String = match ev_state {
            TruckButtonState::Off => "[ ] Unknown\n",
            TruckButtonState::Pressed => "[+] Found\n",
            TruckButtonState::Discard => "[-] Discarded\n",
        }
        .into();

        let status_color = match ev_state {
            TruckButtonState::Off => colors::INVENTORY_STATS_COLOR,
            TruckButtonState::Pressed => Color::GREEN.with_a(0.4),
            TruckButtonState::Discard => Color::RED.with_a(0.4),
        };

        Self {
            title,
            status,
            status_color,
            help_text,
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        update_evidence_ui
            .run_if(in_state(root::GameState::None).and_then(in_state(root::State::InGame))),
    )
    .add_systems(
        Update,
        keyboard_evidence
            .run_if(in_state(root::GameState::None).and_then(in_state(root::State::InGame))),
    );
}
