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
                flex_grow: 0.2,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    // max_height: Val::Percent(90.0),
                    margin: UiRect::bottom(Val::Px(2.0)),
                    aspect_ratio: Some(16.0 / 9.0),
                    flex_grow: 0.5,
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
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::percent(
                    colxrow.0,
                    99.0 / colxrow.0 as f32,
                ),
                grid_template_rows: RepeatedGridTrack::flex(colxrow.1, 1.0),
                column_gap: Val::Percent(0.5),
                row_gap: Val::Percent(0.5),
                padding: UiRect::all(Val::Percent(0.5)),
                flex_grow: 1.0,
                flex_basis: Val::Percent(90.0),
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

pub fn grid_img_text2(
    parent: &mut ChildBuilder<'_>,
    font: &Handle<Font>,
    colxrow: (u16, u16),
    contents: Vec<(&Handle<Image>, impl Into<String>)>,
) {
    let mut rows = vec![];

    for _ in 0..colxrow.1 {
        rows.push(GridTrack::flex(1.0));
        rows.push(GridTrack::flex(0.3));
    }

    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(96.0),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::percent(
                    colxrow.0,
                    99.0 / colxrow.0 as f32,
                ),
                grid_template_rows: rows,
                column_gap: Val::Percent(0.5),
                row_gap: Val::Percent(0.2),
                padding: UiRect::all(Val::Percent(0.2)),
                overflow: Overflow::clip(),
                flex_grow: 1.0,
                flex_basis: Val::Percent(90.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            let contents: Vec<(&Handle<Image>, String)> =
                contents.into_iter().map(|(a, b)| (a, b.into())).collect();
            let rows = colxrow.1;
            let cols = colxrow.0;
            for row in 0..rows {
                for (img, _text) in &contents[(row * cols) as usize..(row * cols + cols) as usize] {
                    parent.spawn(ImageBundle {
                        style: Style {
                            // max_width: Val::Percent(100.0),
                            max_height: Val::Percent(100.0),
                            margin: UiRect::bottom(Val::Px(2.0)),
                            aspect_ratio: Some(21.0 / 9.0),
                            flex_grow: 1.0,
                            ..default()
                        },
                        image: (*img).clone().into(),
                        ..default()
                    });
                }
                for (_img, text) in &contents[(row * cols) as usize..(row * cols + cols) as usize] {
                    parent.spawn(
                        TextBundle::from_section(
                            text,
                            TextStyle {
                                font: font.clone(),
                                font_size: 16.0 * UI_SCALE,
                                color: Color::WHITE,
                            },
                        )
                        .with_style(Style {
                            flex_grow: 0.1,
                            margin: UiRect::all(Val::Px(4.0)),
                            ..Default::default()
                        }),
                    );
                }
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
                    font_size: 32.0 * UI_SCALE,
                    color: Color::WHITE,
                },
            ));

            // Premise Text
            parent.spawn(TextBundle::from_section(
                subtitle,
                TextStyle {
                    font: handles.fonts.chakra.w400_regular.clone(),
                    font_size: 18.0 * UI_SCALE,
                    color: Color::WHITE,
                },
            ));
        });
    parent.spawn(NodeBundle {
        style: Style {
            flex_grow: 0.2,
            flex_basis: Val::Px(10.0),
            ..default()
        },
        ..default()
    });
}

pub fn summary_text(parent: &mut ChildBuilder, handles: &GameAssets, summary: impl Into<String>) {
    parent.spawn(NodeBundle {
        style: Style {
            flex_grow: 0.2,
            flex_basis: Val::Px(0.0),
            ..default()
        },
        ..default()
    });

    parent
        .spawn(NodeBundle {
            style: Style {
                // Occupy full width
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                margin: UiRect::all(Val::Percent(0.5)),
                flex_grow: 0.0,
                flex_basis: Val::Percent(1.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Summary Text
            parent.spawn(TextBundle::from_section(
                summary,
                TextStyle {
                    font: handles.fonts.chakra.w400_regular.clone(),
                    font_size: 18.0 * UI_SCALE,
                    color: Color::WHITE,
                },
            ));
        });
}
