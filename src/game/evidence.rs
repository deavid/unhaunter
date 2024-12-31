use super::GameConfig;
use crate::platform::plt::UI_SCALE;
use crate::{
    colors,
    gear::playergear::PlayerGear,
    ghost_definitions::Evidence,
    player::PlayerSprite,
    root::{self, GameAssets},
    truck::uibutton::{TruckButtonState, TruckButtonType, TruckUIButton},
};
use bevy::color::palettes::css;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct EvidenceUI;

pub fn setup_ui_evidence(parent: &mut ChildBuilder, handles: &GameAssets) {
    parent
        .spawn((
            Text::new(""),
             TextFont {
                    font: handles.fonts.chakra.w400_regular.clone(),
                    font_size: 22.0 * UI_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                 },
             TextColor(colors::INVENTORY_STATS_COLOR.with_alpha(1.0)),
            TextLayout::default(),
             Node::default(),
            EvidenceUI,
        ))
        .with_children(|parent| {
            parent
                .spawn(TextSpan::new("Freezing temps:"))
                .insert(TextFont {
                    font: handles.fonts.chakra.w400_regular.clone(),
                    font_size: 22.0 * UI_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                 })
                .insert(TextColor(colors::INVENTORY_STATS_COLOR.with_alpha(1.0)));
            parent
                .spawn(TextSpan::new(" [+] Evidence Found\n"))
                .insert(TextFont {
                    font: handles.fonts.victormono.w600_semibold.clone(),
                    font_size: 20.0 * UI_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                })
                .insert(TextColor(css::GREEN.with_alpha(0.4).into()));
            parent
                .spawn(TextSpan::new(
                    "The ghost and the breach will make the ambient colder.\nSome ghosts will make the temperature drop below 0.0ºC.",
                ))
                .insert(TextFont {
                    font: handles.fonts.chakra.w300_light.clone(),
                    font_size: 20.0 * UI_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                 })
                .insert(TextColor(colors::INVENTORY_STATS_COLOR));
        });
}

pub fn update_evidence_ui(
    gc: Res<GameConfig>,
    q_gear: Query<(&PlayerSprite, &PlayerGear)>,
    mut qs: Query<Entity, With<EvidenceUI>>,
    interaction_query: Query<&TruckUIButton, With<Button>>,
    mut writer: TextUiWriter,
) {
    for (ps, playergear) in q_gear.iter() {
        if gc.player_id == ps.id {
            for txt_entity in qs.iter_mut() {
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
                if let Some((_entity, _depth, mut text, _font, _color)) = writer.get(txt_entity, 0)
                {
                    if *text != status.title {
                        *text = status.title;
                    }
                }
                if let Some((_entity, _depth, mut text, _font, mut color)) =
                    writer.get(txt_entity, 1)
                {
                    if *text != status.status {
                        *text = status.status;
                        *color = TextColor(status.status_color);
                    }
                }
                if let Some((_entity, _depth, mut text, _font, _color)) = writer.get(txt_entity, 2)
                {
                    if *text != status.help_text {
                        *text = status.help_text;
                    }
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
        let status_color: Color = match ev_state {
            TruckButtonState::Off => colors::INVENTORY_STATS_COLOR,
            TruckButtonState::Pressed => css::GREEN.with_alpha(0.4).into(),
            TruckButtonState::Discard => css::RED.with_alpha(0.4).into(),
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
            .run_if(in_state(root::GameState::None).and(in_state(root::State::InGame))),
    )
    .add_systems(
        Update,
        keyboard_evidence
            .run_if(in_state(root::GameState::None).and(in_state(root::State::InGame))),
    );
}
