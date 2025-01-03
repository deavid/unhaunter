use uncore::colors;
use uncore::components::game_ui::EvidenceUI;
use uncore::platform::plt::FONT_SCALE;
use uncore::types::evidence::Evidence;

pub use uncore::types::evidence_status::EvidenceStatus;

use super::GameConfig;
use crate::{
    player::PlayerSprite,
    truck::uibutton::{TruckButtonState, TruckButtonType, TruckUIButton},
    uncore_root::{self, GameAssets},
};
use bevy::color::palettes::css;
use bevy::prelude::*;
use ungear::components::playergear::PlayerGear;

pub fn setup_ui_evidence(parent: &mut ChildBuilder, handles: &GameAssets) {
    parent
        .spawn((
            Text::default(),
            TextFont {
                font: handles.fonts.chakra.w400_regular.clone(),
                font_size: 22.0 * FONT_SCALE,
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
                    font_size: 22.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                 })
                .insert(TextColor(colors::INVENTORY_STATS_COLOR.with_alpha(1.0)));
            parent
                .spawn(TextSpan::new(" [+] Evidence Found\n"))
                .insert(TextFont {
                    font: handles.fonts.victormono.w600_semibold.clone(),
                    font_size: 20.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                })
                .insert(TextColor(css::GREEN.with_alpha(1.0).into()));
            parent
                .spawn(TextSpan::new(
                    "The ghost and the breach will make the ambient colder.\nSome ghosts will make the temperature drop below 0.0ºC.",
                ))
                .insert(TextFont {
                    font: handles.fonts.chakra.w300_light.clone(),
                    font_size: 20.0 * FONT_SCALE,
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
                if let Some((_entity, _depth, mut text, _font, _color)) = writer.get(txt_entity, 1)
                {
                    if *text != status.title {
                        *text = status.title;
                    }
                }
                if let Some((_entity, _depth, mut text, _font, mut color)) =
                    writer.get(txt_entity, 2)
                {
                    if *text != status.status {
                        *text = status.status;
                        *color = TextColor(status.status_color);
                    }
                }
                if let Some((_entity, _depth, mut text, _font, _color)) = writer.get(txt_entity, 3)
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

pub fn app_setup(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        update_evidence_ui.run_if(
            in_state(uncore_root::GameState::None).and(in_state(uncore_root::AppState::InGame)),
        ),
    )
    .add_systems(
        Update,
        keyboard_evidence.run_if(
            in_state(uncore_root::GameState::None).and(in_state(uncore_root::AppState::InGame)),
        ),
    );
}
