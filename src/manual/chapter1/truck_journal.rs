use bevy::prelude::*;

use crate::root::GameAssets;

pub fn draw_truck_journal_page(parent: &mut ChildBuilder, handles: &GameAssets) {
    parent.spawn(TextBundle::from_section(
        "Learn how to move: WASD. This is the basic controls page.",
        TextStyle {
            font: handles.fonts.londrina.w300_light.clone(),
            font_size: 38.0,
            color: Color::WHITE,
        },
    ));
}