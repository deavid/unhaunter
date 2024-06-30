use bevy::prelude::*;

use crate::root::GameAssets;

pub fn draw_truck_journal_page(parent: &mut ChildBuilder, handles: &GameAssets) {
    parent.spawn(TextBundle::from_section(
        "Use the truck journal to record the evidence you've gathered. The journal will help you narrow down the possible ghost types and craft the correct repellent.",
        TextStyle {
            font: handles.fonts.londrina.w300_light.clone(),
            font_size: 38.0,
            color: Color::WHITE,
        },
    ));
}
