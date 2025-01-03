//! This module provides utility functions for creating and managing the UI elements
//! of the in-game manual in Unhaunter.  It includes functions for drawing common
//! UI components such as headers, grids, text sections, buttons, and other elements.
//! These functions are designed to be shared between different manual modes (user-requested
//! and pre-play tutorial) to promote code reuse and maintain consistency across the
//! manual's interface.  The functions utilize Bevy's UI system to create the
//! visual elements of the manual efficiently.
use uncore::colors;
use uncore::platform::plt::FONT_SCALE;

use crate::uncore_root::GameAssets;

use bevy::prelude::*;

pub fn grid_img_text2(
    parent: &mut ChildBuilder<'_>,
    regular_font: &Handle<Font>,
    bold_font: &Handle<Font>,
    colxrow: (u16, u16),
    contents: Vec<(&Handle<Image>, impl Into<String>)>,
) {
    let font_regular = TextFont {
        font: regular_font.clone(),
        font_size: 16.0 * FONT_SCALE,
        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
    };
    let font_bold = TextFont {
        font: bold_font.clone(),
        font_size: 16.0 * FONT_SCALE,
        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
    };
    let color_regular = TextColor(colors::DIALOG_TEXT_COLOR);
    let color_bold = TextColor(colors::DIALOG_BOLD_TEXT_COLOR);

    let mut rows = vec![];

    for _ in 0..colxrow.1 {
        rows.push(GridTrack::flex(1.0));
        rows.push(GridTrack::flex(0.3));
    }

    parent
        .spawn(Node {
            width: Val::Percent(96.0),
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::percent(colxrow.0, 99.0 / colxrow.0 as f32),
            grid_template_rows: rows,
            column_gap: Val::Percent(0.5),
            row_gap: Val::Percent(0.2),
            padding: UiRect::all(Val::Percent(0.2)),
            overflow: Overflow::clip(),
            flex_grow: 1.0,
            flex_basis: Val::Percent(90.0),
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            ..default()
        })
        .with_children(|parent| {
            let contents: Vec<(&Handle<Image>, String)> =
                contents.into_iter().map(|(a, b)| (a, b.into())).collect();
            let rows = colxrow.1;
            let cols = colxrow.0;
            for row in 0..rows {
                for (img, text) in &contents[(row * cols) as usize..(row * cols + cols) as usize] {
                    if text == "N/A" {
                        parent.spawn(Node::default());
                        continue;
                    }
                    parent
                        .spawn(ImageNode {
                            image: (*img).clone(),
                            ..default()
                        })
                        .insert(Node {
                            max_height: Val::Percent(100.0),
                            margin: UiRect::bottom(Val::Px(2.0)),
                            aspect_ratio: Some(21.0 / 9.0),
                            flex_grow: 1.0,
                            ..default()
                        });
                }
                for (_img, text) in &contents[(row * cols) as usize..(row * cols + cols) as usize] {
                    if text == "N/A" {
                        parent.spawn(Node::default());
                        continue;
                    }
                    let mut layout = parent.spawn((TextLayout::default(), Text::default()));
                    for (n, subtext) in text.split('*').enumerate() {
                        let (font, color) = if n % 2 == 0 {
                            (font_regular.clone(), color_regular)
                        } else {
                            (font_bold.clone(), color_bold)
                        };
                        layout.with_child((TextSpan::new(subtext), font, color));
                    }
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
        .spawn(Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|parent| {
            // Headline
            parent
                .spawn(Text::new(title))
                .insert(TextFont {
                    font: handles.fonts.londrina.w300_light.clone(),
                    font_size: 32.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                })
                .insert(TextColor(Color::WHITE));

            // Premise Text
            parent
                .spawn(Text::new(subtitle))
                .insert(TextFont {
                    font: handles.fonts.chakra.w400_regular.clone(),
                    font_size: 18.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                })
                .insert(TextColor(Color::WHITE));
        });
    parent.spawn(Node {
        flex_grow: 0.2,
        flex_basis: Val::Px(10.0),
        ..default()
    });
}

pub fn summary_text(parent: &mut ChildBuilder, handles: &GameAssets, summary: impl Into<String>) {
    parent.spawn(Node {
        flex_grow: 0.2,
        flex_basis: Val::Px(0.0),
        ..default()
    });

    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            margin: UiRect::all(Val::Percent(0.5)),
            flex_grow: 0.0,
            flex_basis: Val::Percent(1.0),
            ..default()
        })
        .with_children(|parent| {
            // Summary Text
            parent
                .spawn(Text::new(summary))
                .insert(TextFont {
                    font: handles.fonts.chakra.w400_regular.clone(),
                    font_size: 18.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                })
                .insert(TextColor(Color::WHITE));
        });
}
