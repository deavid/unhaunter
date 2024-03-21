use bevy::prelude::*;

use crate::{colors, game::GameConfig, player::PlayerSprite, root};

const MARGIN_PERCENT: f32 = 0.5;
const TEXT_MARGIN: UiRect = UiRect::percent(2.0, 0.0, 0.0, 0.0);

#[derive(Component, Debug)]
pub struct SanityText;

pub fn setup_sanity_ui(p: &mut ChildBuilder, handles: &root::GameAssets) {
    let title = TextBundle::from_section(
        "Sanity",
        TextStyle {
            font: handles.fonts.londrina.w300_light.clone(),
            font_size: 35.0,
            color: colors::TRUCKUI_ACCENT_COLOR,
        },
    )
    .with_style(Style {
        height: Val::Px(40.0),
        ..default()
    });

    p.spawn(title);
    // Sanity contents
    p.spawn(NodeBundle {
        border_color: colors::TRUCKUI_ACCENT_COLOR.into(),
        style: Style {
            border: UiRect::top(Val::Px(2.0)),
            height: Val::Px(0.0),
            ..default()
        },
        ..default()
    });
    let mut p1_sanity = TextBundle::from_section(
        "Player 1: 90% Sanity",
        TextStyle {
            font: handles.fonts.chakra.w300_light.clone(),
            font_size: 25.0,
            color: colors::TRUCKUI_TEXT_COLOR,
        },
    );
    p1_sanity.style.margin = TEXT_MARGIN;

    p.spawn(p1_sanity).insert(SanityText);

    p.spawn(NodeBundle {
        style: Style {
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Percent(MARGIN_PERCENT),
            flex_grow: 1.0,
            ..default()
        },
        ..default()
    });
}

pub fn update_sanity(
    gc: Res<GameConfig>,
    qp: Query<&PlayerSprite>,
    mut qst: Query<&mut Text, With<SanityText>>,
) {
    for player in &qp {
        if player.id != gc.player_id {
            continue;
        }
        for mut text in &mut qst {
            let new_sanity_text = format!("Player 1:\n  {:.0}% Sanity", player.sanity());
            if new_sanity_text != text.sections[0].value {
                text.sections[0].value = new_sanity_text;
            }
        }
    }
}
