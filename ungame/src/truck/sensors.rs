use crate::platform::plt::{UI_SCALE, FONT_SCALE};
use crate::{colors, root};
use bevy::prelude::*;

const MARGIN_PERCENT: f32 = 0.5 * UI_SCALE;
const TEXT_MARGIN: UiRect = UiRect::percent(2.0 * UI_SCALE, 0.0, 0.0, 0.0);

pub fn setup_sensors_ui(p: &mut ChildBuilder, handles: &root::GameAssets) {
    let title = (
        Text::new("Sensors"),
        TextFont {
            font: handles.fonts.londrina.w300_light.clone(),
            font_size: 35.0 * FONT_SCALE,
            ..default()
        },
        TextColor(colors::TRUCKUI_ACCENT_COLOR),
        Node {
            height: Val::Px(40.0 * UI_SCALE),

            ..default()
        },
    );
    p.spawn(title);

    // Sensors contents
    p.spawn(Node {
        border: UiRect::top(Val::Px(2.0 * UI_SCALE)),
        height: Val::Px(0.0),
        ..default()
    })
    .insert(BorderColor(colors::TRUCKUI_ACCENT_COLOR));

    let sensor1 = (
        Text::new("No Sensors"),
        TextFont {
            font: handles.fonts.chakra.w300_light.clone(),
            font_size: 25.0 * FONT_SCALE,
            ..default()
        },
        TextColor(colors::TRUCKUI_TEXT_COLOR),
        Node {
            margin: TEXT_MARGIN,
            ..default()
        },
    );
    p.spawn(sensor1);
    p.spawn(Node {
        justify_content: JustifyContent::FlexStart,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Percent(MARGIN_PERCENT),
        flex_grow: 1.0,
        ..default()
    });
}
