use crate::{manual::ManualPageData, root::GameAssets};
use bevy::prelude::*;

pub fn draw(parent: &mut ChildBuilder, handles: &GameAssets) {
    parent.spawn(
        TextBundle::from_section(
            "Use the truck journal to record the evidence you've gathered. The journal will help you narrow down the possible ghost types and craft the correct repellent.",
            TextStyle {
                font: handles.fonts.londrina.w300_light.clone(),
                font_size: 38.0,
                color: Color::WHITE,
            },
        ),
    );
}

pub fn create_manual_page() -> ManualPageData {
    ManualPageData {
        title: "Truck Journal".into(),
        subtitle: "Identifying the Ghost".into(),
        draw_fn: draw,
    }
}
