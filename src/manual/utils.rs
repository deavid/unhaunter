use bevy::prelude::*;

use crate::platform::plt::UI_SCALE;

pub fn image_text(
    parent: &mut ChildBuilder<'_>,
    image: &Handle<Image>,
    font: &Handle<Font>,
    text: impl Into<String>,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    max_width: Val::Percent(90.0),
                    max_height: Val::Percent(80.0),
                    margin: UiRect::all(Val::Px(2.0)),
                    aspect_ratio: Some(16.0 / 9.0),
                    ..default()
                },
                image: image.clone().into(),
                ..default()
            });
            parent.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font: font.clone(),
                    font_size: 18.0 * UI_SCALE,
                    color: Color::WHITE,
                },
            ));
        });
}
