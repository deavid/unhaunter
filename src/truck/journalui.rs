use bevy::prelude::*;

use crate::{colors, difficulty::CurrentDifficulty, ghost_definitions, root};

use super::{uibutton::TruckButtonType, TruckUIGhostGuess};
use crate::platform::plt::UI_SCALE;

const MARGIN_PERCENT: f32 = 0.5 * UI_SCALE;
const MARGIN: UiRect = UiRect::percent(
    MARGIN_PERCENT,
    MARGIN_PERCENT,
    MARGIN_PERCENT,
    MARGIN_PERCENT,
);

pub fn setup_journal_ui(
    p: &mut ChildBuilder,
    handles: &root::GameAssets,
    difficulty: &CurrentDifficulty,
) {
    // Journal contents

    p.spawn(
        TextBundle::from_section(
            "Select evidence:",
            TextStyle {
                font: handles.fonts.chakra.w300_light.clone(),
                font_size: 25.0 * UI_SCALE,
                color: colors::TRUCKUI_TEXT_COLOR,
            },
        )
        .with_style(Style {
            margin: UiRect::all(Val::Px(4.0 * UI_SCALE)),
            ..default()
        }),
    );

    // Evidence selection
    p.spawn(NodeBundle {
        style: Style {
            justify_content: JustifyContent::FlexStart,
            // flex_direction: FlexDirection::Row,
            // flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(4.0 * UI_SCALE),
            column_gap: Val::Px(4.0 * UI_SCALE),
            display: Display::Grid,
            grid_template_columns: vec![
                GridTrack::auto(),
                GridTrack::auto(),
                GridTrack::auto(),
                GridTrack::auto(),
            ],
            grid_template_rows: vec![GridTrack::auto(), GridTrack::auto()],

            ..default()
        },
        ..default()
    })
    .with_children(|evblock| {
        for evidence in ghost_definitions::Evidence::all() {
            evblock
                .spawn(ButtonBundle {
                    style: Style {
                        min_height: Val::Px(20.0 * UI_SCALE),
                        border: UiRect::all(Val::Px(0.9)),
                        align_content: AlignContent::Center,
                        justify_content: JustifyContent::Center,
                        display: Display::Grid,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Percent(MARGIN_PERCENT)),
                        padding: UiRect::all(Val::Px(4.0 * UI_SCALE)),
                        ..default()
                    },
                    ..default()
                })
                .insert(TruckButtonType::Evidence(evidence).into_component())
                .with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        evidence.name(),
                        TextStyle {
                            font: handles.fonts.titillium.w400_regular.clone(),
                            font_size: 22.0 * UI_SCALE,
                            ..default()
                        },
                    ));
                });
        }
    });
    // ---- Ghost guess
    p.spawn(NodeBundle {
        style: Style {
            margin: UiRect::all(Val::Px(4.0 * UI_SCALE)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::End,
            column_gap: Val::Percent(MARGIN_PERCENT),
            flex_basis: Val::Px(50.0 * UI_SCALE),
            flex_grow: 0.5,
            flex_shrink: 0.0,
            ..default()
        },
        ..default()
    })
    .with_children(|guess| {
        guess.spawn(
            TextBundle::from_section(
                "Possible ghost with the selected evidence:",
                TextStyle {
                    font: handles.fonts.chakra.w300_light.clone(),
                    font_size: 25.0 * UI_SCALE,
                    color: colors::TRUCKUI_TEXT_COLOR,
                },
            )
            .with_style(Style {
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..default()
            }),
        );
    });

    // Ghost selection
    p.spawn(NodeBundle {
        style: Style {
            justify_content: JustifyContent::FlexStart,
            // row_gap: Val::Px(4.0),
            // column_gap: Val::Px(4.0),
            display: Display::Grid,
            grid_template_columns: vec![
                GridTrack::auto(),
                GridTrack::auto(),
                GridTrack::auto(),
                GridTrack::auto(),
                GridTrack::auto(),
            ],
            grid_auto_rows: GridTrack::flex(1.0),
            flex_grow: 1.0,
            ..default()
        },
        background_color: colors::TRUCKUI_BGCOLOR.into(),
        ..default()
    })
    .with_children(|ghost_selection| {
        for ghost_type in difficulty.0.ghost_set.as_vec() {
            ghost_selection
                .spawn(ButtonBundle {
                    style: Style {
                        min_height: Val::Px(20.0 * UI_SCALE),
                        border: UiRect::all(Val::Px(0.9)),
                        align_content: AlignContent::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::new(
                            Val::Px(5.0),
                            Val::Px(2.0),
                            Val::Px(0.0),
                            Val::Px(2.0),
                        ),
                        display: Display::Grid,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .insert(TruckButtonType::Ghost(ghost_type).into_component())
                .with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        ghost_type.name(),
                        TextStyle {
                            font: handles.fonts.titillium.w400_regular.clone(),
                            font_size: 22.0 * UI_SCALE,
                            ..default()
                        },
                    ));
                });
        }
    });
    // --- Ghost selected
    p.spawn(NodeBundle {
        style: Style {
            margin: UiRect::all(Val::Px(4.0 * UI_SCALE)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::End,
            column_gap: Val::Percent(MARGIN_PERCENT),
            flex_basis: Val::Px(50.0 * UI_SCALE),
            flex_grow: 0.5,
            flex_shrink: 0.0,
            ..default()
        },

        ..default()
    })
    .with_children(|guess| {
        guess.spawn(
            TextBundle::from_section(
                "With the above evidence we believe the ghost is:",
                TextStyle {
                    font: handles.fonts.chakra.w300_light.clone(),
                    font_size: 25.0 * UI_SCALE,
                    color: colors::TRUCKUI_TEXT_COLOR,
                },
            )
            .with_style(Style {
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..default()
            }),
        );
        let ghost_guess = TextBundle::from_section(
            "-- Unknown --",
            TextStyle {
                font: handles.fonts.titillium.w600_semibold.clone(),
                font_size: 28.0 * UI_SCALE,
                color: colors::TRUCKUI_TEXT_COLOR,
            },
        );
        guess
            .spawn(NodeBundle {
                background_color: colors::TRUCKUI_BGCOLOR.into(),
                style: Style {
                    padding: UiRect::all(Val::Px(4.0 * UI_SCALE)),
                    flex_basis: Val::Px(300.0 * UI_SCALE),
                    flex_grow: 0.0,
                    flex_shrink: 0.0,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|node| {
                node.spawn(ghost_guess).insert(TruckUIGhostGuess);
            });
    });

    // ---- Synthesis of Unhaunter essence
    p.spawn(ButtonBundle {
        style: Style {
            min_height: Val::Px(30.0 * UI_SCALE),
            width: Val::Percent(50.0),
            border: MARGIN,
            align_self: AlignSelf::Center,
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            margin: MARGIN,
            ..default()
        },
        ..default()
    })
    .insert(TruckButtonType::CraftRepellent.into_component())
    .with_children(|btn| {
        btn.spawn(TextBundle::from_section(
            "Craft Unhaunter™ Ghost Repellent",
            TextStyle {
                font: handles.fonts.titillium.w600_semibold.clone(),
                font_size: 32.0 * UI_SCALE,
                ..default()
            },
        ));
    });
}
