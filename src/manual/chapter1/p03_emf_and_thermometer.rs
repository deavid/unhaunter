use crate::root::GameAssets;
use bevy::prelude::*;

pub fn draw_emf_and_thermometer_page(parent: &mut ChildBuilder, handles: &GameAssets) {
    parent.spawn(
        TextBundle::from_section(
            "The EMF Reader detects fluctuations in the electromagnetic field, often indicating a ghost's presence. The Thermometer helps you find cold spots, which can also signal paranormal activity.",
            TextStyle {
                font: handles.fonts.londrina.w300_light.clone(),
                font_size: 38.0,
                color: Color::WHITE,
            },
        ),
    );
}
