use bevy::prelude::*;

use crate::root::GameAssets;

pub fn draw_introduction_page(parent: &mut ChildBuilder, handles: &GameAssets) {
    parent.spawn(TextBundle::from_section(
        "Welcome to Unhaunter! This is the introduction page.",
        TextStyle {
            font: handles.fonts.londrina.w300_light.clone(),
            font_size: 38.0,
            color: Color::WHITE,
        },
    ));
}
