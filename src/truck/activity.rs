use crate::platform::plt::UI_SCALE;
use crate::{colors, root};
use bevy::prelude::*;

const MARGIN_PERCENT: f32 = 0.5 * UI_SCALE;
const TEXT_MARGIN: UiRect = UiRect::percent(2.0 * UI_SCALE, 0.0, 0.0, 0.0);

pub fn setup_activity_ui(p: &mut ChildBuilder, handles: &root::GameAssets) {
    let title = TextBundle::from_section(
        "Activity",
        TextStyle {
            font: handles.fonts.londrina.w300_light.clone(),
            font_size: 35.0 * UI_SCALE,
            color: colors::TRUCKUI_ACCENT_COLOR,
        },
    )
    .with_style(Style {
        height: Val::Px(40.0 * UI_SCALE),
        ..default()
    });
    p.spawn(title);

    // Activity contents
    p.spawn(NodeBundle {
        border_color: colors::TRUCKUI_ACCENT_COLOR.into(),
        style: Style {
            border: UiRect::top(Val::Px(2.0 * UI_SCALE)),
            height: Val::Px(0.0),
            ..default()
        },
        ..default()
    });
    let mut sample_text = TextBundle::from_section(
        "Instrumentation broken",
        TextStyle {
            font: handles.fonts.chakra.w300_light.clone(),
            font_size: 25.0 * UI_SCALE,
            color: colors::TRUCKUI_TEXT_COLOR,
        },
    );
    sample_text.style.margin = TEXT_MARGIN;
    p.spawn(sample_text);
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
