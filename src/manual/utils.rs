use crate::platform::plt::UI_SCALE;
use crate::root::GameAssets;

use bevy::prelude::*;

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

pub fn grid_img_text(
    parent: &mut ChildBuilder<'_>,
    font: &Handle<Font>,
    colxrow: (u16, u16),
    contents: Vec<(&Handle<Image>, impl Into<String>)>,
) {
    // Gameplay Loop Section (3x2 Grid)
    parent
        .spawn(NodeBundle {
            style: Style {
                // Occupy full width
                width: Val::Percent(100.0),
                height: Val::Percent(70.0),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(colxrow.0, 1.0),
                grid_template_rows: RepeatedGridTrack::percent(colxrow.1, 60.0),
                column_gap: Val::Percent(4.0),
                row_gap: Val::Percent(4.0),
                padding: UiRect::all(Val::Percent(2.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for (img, text) in contents {
                image_text(parent, img, font, text);
            }
        });
}

pub fn header(
    parent: &mut ChildBuilder,
    handles: &GameAssets,
    title: impl Into<String>,
    subtitle: impl Into<String>,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                // Occupy full width
                width: Val::Percent(100.0),
                height: Val::Percent(10.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Headline
            parent.spawn(TextBundle::from_section(
                title,
                TextStyle {
                    font: handles.fonts.londrina.w300_light.clone(),
                    font_size: 48.0 * UI_SCALE,
                    color: Color::WHITE,
                },
            ));

            // Premise Text
            parent.spawn(TextBundle::from_section(
                subtitle,
                TextStyle {
                    font: handles.fonts.chakra.w400_regular.clone(),
                    font_size: 24.0 * UI_SCALE,
                    color: Color::WHITE,
                },
            ));
        });
}
