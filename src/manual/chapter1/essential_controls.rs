use bevy::prelude::*;

use crate::root::GameAssets;

pub fn draw_essential_controls_page(parent: &mut ChildBuilder, handles: &GameAssets) {
    parent.spawn(TextBundle::from_section(
        "Move with WASD keys. Interact with doors, switches, and lights using 'E'. Toggle your flashlight with 'TAB'. Use your equipped gear with 'R'.",
        TextStyle {
            font: handles.fonts.londrina.w300_light.clone(),
            font_size: 38.0,
            color: Color::WHITE,
        },
    ));
}
