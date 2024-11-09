use crate::{manual::ManualPageData, root::GameAssets};
use bevy::prelude::*;

pub fn draw(parent: &mut ChildBuilder, handles: &GameAssets) {
    parent.spawn(
        TextBundle::from_section(
            "Once you've crafted the repellent, confront the ghost and use it to banish it. Return to your truck and click 'End Mission' to complete the investigation.",
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
        title: "Expelling the Ghost".into(),
        subtitle: "Banishing the Spirit".into(),
        draw_fn: draw,
    }
}
