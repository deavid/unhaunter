use bevy::prelude::*;

use crate::{colors, root};

pub fn setup_loadout_ui(p: &mut ChildBuilder, handles: &root::GameAssets) {
    p.spawn(
        TextBundle::from_section(
            "Player Inventory:",
            TextStyle {
                font: handles.fonts.chakra.w300_light.clone(),
                font_size: 25.0,
                color: colors::TRUCKUI_TEXT_COLOR,
            },
        )
        .with_style(Style {
            margin: UiRect::all(Val::Px(4.0)),
            ..default()
        }),
    );
    p.spawn(NodeBundle {
        style: Style {
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Column,
            flex_grow: 0.4,
            min_height: Val::Px(100.0),
            ..default()
        },
        ..default()
    });
    p.spawn(
        TextBundle::from_section(
            "Van Inventory:",
            TextStyle {
                font: handles.fonts.chakra.w300_light.clone(),
                font_size: 25.0,
                color: colors::TRUCKUI_TEXT_COLOR,
            },
        )
        .with_style(Style {
            margin: UiRect::all(Val::Px(4.0)),
            ..default()
        }),
    );
    p.spawn(NodeBundle {
        background_color: colors::TRUCKUI_BGCOLOR.into(),
        style: Style {
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Column,
            flex_grow: 0.7,
            min_height: Val::Px(200.0),
            ..default()
        },
        ..default()
    });
}
