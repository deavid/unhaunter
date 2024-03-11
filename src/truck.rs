use std::ops::Mul;

use bevy::prelude::*;
use bevy::{app::App, utils::HashSet};

use crate::colors;
use crate::game::GameConfig;
use crate::gear::playergear::PlayerGear;
use crate::player::PlayerSprite;
use crate::{
    ghost_definitions::{self, Evidence, GhostType},
    materials::{self, UIPanelMaterial},
    root,
};

#[derive(Component, Debug)]
pub struct TruckUI;

#[derive(Clone, Debug, Event)]
pub enum TruckUIEvent {
    EndMission,
    ExitTruck,
    CraftRepellent,
}

#[derive(Component, Debug)]
pub struct SanityText;

#[derive(Component, Debug)]
pub struct TruckUIGhostGuess;

#[derive(Debug, Resource, Default)]
pub struct GhostGuess {
    pub ghost_type: Option<GhostType>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TruckButtonType {
    Evidence(ghost_definitions::Evidence),
    Ghost(ghost_definitions::GhostType),
    CraftRepellent,
    ExitTruck,
    EndMission,
}

impl TruckButtonType {
    pub fn into_component(self) -> TruckUIButton {
        TruckUIButton::from(self)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TruckButtonState {
    Off,
    Pressed,
    Discard,
}
#[derive(Component, Debug)]
pub struct TruckUIButton {
    status: TruckButtonState,
    class: TruckButtonType,
    disabled: bool,
}

impl TruckUIButton {
    pub fn pressed(&mut self) -> Option<TruckUIEvent> {
        match self.class {
            TruckButtonType::Evidence(_) | TruckButtonType::Ghost(_) => {
                self.status = match self.status {
                    TruckButtonState::Off => TruckButtonState::Pressed,
                    TruckButtonState::Pressed => TruckButtonState::Discard,
                    TruckButtonState::Discard => TruckButtonState::Off,
                };
                None
            }
            TruckButtonType::CraftRepellent => Some(TruckUIEvent::CraftRepellent),
            TruckButtonType::ExitTruck => Some(TruckUIEvent::ExitTruck),
            TruckButtonType::EndMission => Some(TruckUIEvent::EndMission),
        }
    }
    pub fn border_color(&self, interaction: Interaction) -> Color {
        match self.class {
            TruckButtonType::Evidence(_) => match interaction {
                Interaction::Pressed => colors::TRUCKUI_ACCENT3_COLOR,
                Interaction::Hovered => colors::TRUCKUI_TEXT_COLOR,
                Interaction::None => colors::TRUCKUI_ACCENT2_COLOR,
            },
            TruckButtonType::Ghost(_) => match interaction {
                Interaction::Pressed => colors::TRUCKUI_ACCENT3_COLOR,
                Interaction::Hovered => colors::TRUCKUI_ACCENT_COLOR,
                Interaction::None => Color::NONE,
            },
            TruckButtonType::ExitTruck | TruckButtonType::CraftRepellent => match interaction {
                Interaction::Pressed => colors::BUTTON_EXIT_TRUCK_TXTCOLOR,
                Interaction::Hovered => colors::BUTTON_EXIT_TRUCK_TXTCOLOR,
                Interaction::None => colors::BUTTON_EXIT_TRUCK_FGCOLOR,
            },
            TruckButtonType::EndMission => match interaction {
                Interaction::Pressed => colors::BUTTON_END_MISSION_TXTCOLOR,
                Interaction::Hovered => colors::BUTTON_END_MISSION_TXTCOLOR,
                Interaction::None => colors::BUTTON_END_MISSION_FGCOLOR,
            },
        }
        .mul(
            Color::WHITE
                .with_a(if self.disabled { 0.05 } else { 1.0 })
                .as_rgba_f32(),
        )
    }
    pub fn background_color(&self, interaction: Interaction) -> Color {
        match self.class {
            TruckButtonType::Evidence(_) => match self.status {
                TruckButtonState::Off => colors::TRUCKUI_BGCOLOR,
                TruckButtonState::Pressed => colors::TRUCKUI_ACCENT2_COLOR,
                TruckButtonState::Discard => colors::BUTTON_END_MISSION_FGCOLOR,
            },
            TruckButtonType::Ghost(_) => match self.status {
                TruckButtonState::Off => colors::TRUCKUI_BGCOLOR.with_a(0.0),
                TruckButtonState::Pressed => colors::TRUCKUI_ACCENT2_COLOR,
                TruckButtonState::Discard => colors::BUTTON_END_MISSION_FGCOLOR,
            },
            TruckButtonType::ExitTruck | TruckButtonType::CraftRepellent => match interaction {
                Interaction::Pressed => colors::BUTTON_EXIT_TRUCK_FGCOLOR,
                Interaction::Hovered => colors::BUTTON_EXIT_TRUCK_BGCOLOR,
                Interaction::None => colors::BUTTON_EXIT_TRUCK_BGCOLOR,
            },
            TruckButtonType::EndMission => match interaction {
                Interaction::Pressed => colors::BUTTON_END_MISSION_FGCOLOR,
                Interaction::Hovered => colors::BUTTON_END_MISSION_BGCOLOR,
                Interaction::None => colors::BUTTON_END_MISSION_BGCOLOR,
            },
        }
        .mul(
            Color::WHITE
                .with_a(if self.disabled { 0.05 } else { 1.0 })
                .as_rgba_f32(),
        )
    }

    pub fn text_color(&self, _interaction: Interaction) -> Color {
        match self.class {
            TruckButtonType::Evidence(_) => match self.status {
                TruckButtonState::Pressed => Color::BLACK,
                _ => colors::TRUCKUI_TEXT_COLOR,
            },
            TruckButtonType::Ghost(_) => match self.status {
                TruckButtonState::Pressed => Color::BLACK,
                _ => colors::TRUCKUI_TEXT_COLOR.with_a(0.5),
            },
            TruckButtonType::ExitTruck | TruckButtonType::CraftRepellent => {
                colors::BUTTON_EXIT_TRUCK_TXTCOLOR
            }
            TruckButtonType::EndMission => colors::BUTTON_END_MISSION_TXTCOLOR,
        }
        .mul(
            Color::WHITE
                .with_a(if self.disabled { 0.1 } else { 1.0 })
                .as_rgba_f32(),
        )
    }
}

impl From<TruckButtonType> for TruckUIButton {
    fn from(value: TruckButtonType) -> Self {
        TruckUIButton {
            status: TruckButtonState::Off,
            class: value,
            disabled: false,
        }
    }
}

pub fn setup_ui(
    mut commands: Commands,
    mut materials: ResMut<Assets<materials::UIPanelMaterial>>,
    handles: Res<root::GameAssets>,
) {
    // Load Truck UI
    const MARGIN_PERCENT: f32 = 0.5;
    const TEXT_MARGIN: UiRect = UiRect::percent(2.0, 0.0, 0.0, 0.0);
    const MARGIN: UiRect = UiRect::percent(
        MARGIN_PERCENT,
        MARGIN_PERCENT,
        MARGIN_PERCENT,
        MARGIN_PERCENT,
    );
    commands
        .spawn(NodeBundle {
            background_color: colors::TRUCKUI_BGCOLOR.into(),

            style: Style {
                width: Val::Percent(98.0),
                height: Val::Percent(96.0),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Percent(MARGIN_PERCENT),
                padding: MARGIN,
                margin: MARGIN,
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        })
        .insert(TruckUI)
        .with_children(|parent| {
            // Left column
            parent
                .spawn(NodeBundle {
                    border_color: colors::DEBUG_BCOLOR,
                    style: Style {
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::FlexStart,
                        flex_direction: FlexDirection::Column,
                        min_width: Val::Px(180.0),
                        min_height: Val::Px(10.0),
                        row_gap: Val::Percent(MARGIN_PERCENT),
                        flex_grow: 0.4,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|left_col| {
                    // Top Left - Sanity
                    left_col
                        .spawn(MaterialNodeBundle {
                            material: materials.add(UIPanelMaterial {
                                color: colors::TRUCKUI_PANEL_BGCOLOR,
                            }),

                            style: Style {
                                border: UiRect::all(Val::Px(1.0)),
                                padding: UiRect::left(Val::Percent(MARGIN_PERCENT)),
                                margin: MARGIN,
                                justify_content: JustifyContent::FlexStart,
                                flex_direction: FlexDirection::Column,
                                min_width: Val::Px(10.0),
                                min_height: Val::Px(10.0),
                                flex_grow: 1.0,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|sanity| {
                            let title = TextBundle::from_section(
                                "Sanity",
                                TextStyle {
                                    font: handles.fonts.londrina.w300_light.clone(),
                                    font_size: 35.0,
                                    color: colors::TRUCKUI_ACCENT_COLOR,
                                },
                            )
                            .with_style(Style {
                                height: Val::Px(40.0),
                                ..default()
                            });

                            sanity.spawn(title);
                            // Sanity contents
                            sanity.spawn(NodeBundle {
                                border_color: colors::TRUCKUI_ACCENT_COLOR.into(),
                                style: Style {
                                    border: UiRect::top(Val::Px(2.0)),
                                    height: Val::Px(0.0),
                                    ..default()
                                },
                                ..default()
                            });
                            let mut p1_sanity = TextBundle::from_section(
                                "Player 1: 90% Sanity",
                                TextStyle {
                                    font: handles.fonts.chakra.w300_light.clone(),
                                    font_size: 25.0,
                                    color: colors::TRUCKUI_TEXT_COLOR,
                                },
                            );
                            p1_sanity.style.margin = TEXT_MARGIN;

                            sanity.spawn(p1_sanity).insert(SanityText);

                            sanity.spawn(NodeBundle {
                                style: Style {
                                    justify_content: JustifyContent::FlexStart,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Percent(MARGIN_PERCENT),
                                    flex_grow: 1.0,
                                    ..default()
                                },
                                ..default()
                            });
                        });
                    // Bottom Left - Sensors
                    left_col
                        .spawn(MaterialNodeBundle {
                            material: materials.add(UIPanelMaterial {
                                color: colors::TRUCKUI_PANEL_BGCOLOR,
                            }),

                            style: Style {
                                border: UiRect::all(Val::Px(1.0)),
                                padding: UiRect::left(Val::Percent(MARGIN_PERCENT)),
                                margin: MARGIN,
                                justify_content: JustifyContent::FlexStart,
                                flex_direction: FlexDirection::Column,
                                min_width: Val::Px(10.0),
                                min_height: Val::Px(10.0),
                                flex_grow: 1.0,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|sensors| {
                            let title = TextBundle::from_section(
                                "Sensors",
                                TextStyle {
                                    font: handles.fonts.londrina.w300_light.clone(),
                                    font_size: 35.0,
                                    color: colors::TRUCKUI_ACCENT_COLOR,
                                },
                            )
                            .with_style(Style {
                                height: Val::Px(40.0),
                                ..default()
                            });

                            sensors.spawn(title);
                            // Sensors contents
                            sensors.spawn(NodeBundle {
                                border_color: colors::TRUCKUI_ACCENT_COLOR.into(),
                                style: Style {
                                    border: UiRect::top(Val::Px(2.0)),
                                    height: Val::Px(0.0),
                                    ..default()
                                },
                                ..default()
                            });
                            let mut sensor1 = TextBundle::from_section(
                                "No Sensors",
                                TextStyle {
                                    font: handles.fonts.chakra.w300_light.clone(),
                                    font_size: 25.0,
                                    color: colors::TRUCKUI_TEXT_COLOR,
                                },
                            );
                            sensor1.style.margin = TEXT_MARGIN;

                            sensors.spawn(sensor1);

                            sensors.spawn(NodeBundle {
                                style: Style {
                                    justify_content: JustifyContent::FlexStart,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Percent(MARGIN_PERCENT),
                                    flex_grow: 1.0,
                                    ..default()
                                },
                                ..default()
                            });
                        });
                });
            // Mid content
            parent
                .spawn(MaterialNodeBundle {
                    material: materials.add(UIPanelMaterial {
                        color: colors::TRUCKUI_PANEL_BGCOLOR,
                    }),

                    style: Style {
                        border: UiRect::all(Val::Px(1.0)),
                        padding: UiRect::all(Val::Px(1.0)),
                        min_width: Val::Px(10.0),
                        min_height: Val::Px(10.0),
                        justify_content: JustifyContent::FlexStart,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Percent(MARGIN_PERCENT),
                        flex_grow: 1.0,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|mid_blk| {
                    let title = TextBundle::from_section(
                        "Journal",
                        TextStyle {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 35.0,
                            color: colors::TRUCKUI_ACCENT_COLOR,
                        },
                    )
                    .with_style(Style {
                        height: Val::Px(40.0),
                        ..default()
                    });

                    mid_blk.spawn(title);
                    // Journal contents
                    mid_blk.spawn(NodeBundle {
                        border_color: colors::TRUCKUI_ACCENT_COLOR.into(),
                        style: Style {
                            border: UiRect::top(Val::Px(1.50)),
                            height: Val::Px(0.0),
                            ..default()
                        },
                        ..default()
                    });

                    mid_blk.spawn(
                        TextBundle::from_section(
                            "Select evidence:",
                            TextStyle {
                                font: handles.fonts.chakra.w300_light.clone(),
                                font_size: 25.0,
                                color: colors::TRUCKUI_TEXT_COLOR,
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(4.0)),
                            ..default()
                        }),
                    );

                    // Evidence selection
                    mid_blk
                        .spawn(NodeBundle {
                            style: Style {
                                justify_content: JustifyContent::FlexStart,
                                // flex_direction: FlexDirection::Row,
                                // flex_wrap: FlexWrap::Wrap,
                                row_gap: Val::Px(4.0),
                                column_gap: Val::Px(4.0),
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
                                            min_height: Val::Px(20.0),
                                            border: UiRect::all(Val::Px(0.9)),
                                            align_content: AlignContent::Center,
                                            justify_content: JustifyContent::Center,
                                            display: Display::Grid,
                                            flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::Center,
                                            margin: UiRect::all(Val::Percent(MARGIN_PERCENT)),
                                            padding: UiRect::all(Val::Px(4.0)),
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
                                                font_size: 22.0,
                                                ..default()
                                            },
                                        ));
                                    });
                            }
                        });
                    // ---- Ghost guess
                    mid_blk
                        .spawn(NodeBundle {
                            style: Style {
                                margin: UiRect::all(Val::Px(4.0)),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::End,
                                column_gap: Val::Percent(MARGIN_PERCENT),
                                flex_basis: Val::Px(50.0),
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
                                        font_size: 25.0,
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
                    mid_blk
                        .spawn(NodeBundle {
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
                                ],
                                grid_auto_rows: GridTrack::flex(1.0),
                                flex_grow: 1.0,
                                ..default()
                            },
                            background_color: colors::TRUCKUI_BGCOLOR.into(),
                            ..default()
                        })
                        .with_children(|ghost_selection| {
                            for ghost_type in ghost_definitions::GhostType::all() {
                                ghost_selection
                                    .spawn(ButtonBundle {
                                        style: Style {
                                            min_height: Val::Px(20.0),
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
                                                font_size: 22.0,
                                                ..default()
                                            },
                                        ));
                                    });
                            }
                        });
                    // --- Ghost selected
                    mid_blk
                        .spawn(NodeBundle {
                            style: Style {
                                margin: UiRect::all(Val::Px(4.0)),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::End,
                                column_gap: Val::Percent(MARGIN_PERCENT),
                                flex_basis: Val::Px(50.0),
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
                                        font_size: 25.0,
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
                                    font_size: 28.0,
                                    color: colors::TRUCKUI_TEXT_COLOR,
                                },
                            );
                            guess
                                .spawn(NodeBundle {
                                    background_color: colors::TRUCKUI_BGCOLOR.into(),
                                    style: Style {
                                        padding: UiRect::all(Val::Px(4.0)),
                                        flex_basis: Val::Px(300.0),
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
                    mid_blk
                        .spawn(ButtonBundle {
                            style: Style {
                                min_height: Val::Px(30.0),
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
                                    font_size: 32.0,
                                    ..default()
                                },
                            ));
                        });

                    // ----
                    mid_blk.spawn(NodeBundle {
                        style: Style {
                            justify_content: JustifyContent::FlexStart,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Percent(MARGIN_PERCENT),
                            flex_grow: 1.0,
                            ..default()
                        },
                        ..default()
                    });
                });
            // Right column
            parent
                .spawn(NodeBundle {
                    border_color: colors::DEBUG_BCOLOR,

                    style: Style {
                        border: UiRect::all(Val::Px(1.0)),
                        min_width: Val::Px(10.0),
                        min_height: Val::Px(10.0),
                        justify_content: JustifyContent::FlexStart,
                        row_gap: Val::Percent(MARGIN_PERCENT),
                        flex_direction: FlexDirection::Column,
                        flex_grow: 0.4,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|right_col| {
                    // Top Right - Activity
                    right_col
                        .spawn(MaterialNodeBundle {
                            material: materials.add(UIPanelMaterial {
                                color: colors::TRUCKUI_PANEL_BGCOLOR,
                            }),

                            style: Style {
                                border: UiRect::all(Val::Px(1.0)),
                                padding: UiRect::all(Val::Px(1.0)),
                                margin: MARGIN,
                                row_gap: Val::Percent(MARGIN_PERCENT),
                                flex_direction: FlexDirection::Column,
                                min_width: Val::Px(10.0),
                                min_height: Val::Px(10.0),
                                flex_grow: 1.0,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|activity| {
                            let title = TextBundle::from_section(
                                "Activity",
                                TextStyle {
                                    font: handles.fonts.londrina.w300_light.clone(),
                                    font_size: 35.0,
                                    color: colors::TRUCKUI_ACCENT_COLOR,
                                },
                            )
                            .with_style(Style {
                                height: Val::Px(40.0),
                                ..default()
                            });

                            activity.spawn(title);
                            // Activity contents
                            activity.spawn(NodeBundle {
                                border_color: colors::TRUCKUI_ACCENT_COLOR.into(),
                                style: Style {
                                    border: UiRect::top(Val::Px(2.0)),
                                    height: Val::Px(0.0),
                                    ..default()
                                },
                                ..default()
                            });
                            let mut sample_text = TextBundle::from_section(
                                "Instrumentation broken",
                                TextStyle {
                                    font: handles.fonts.chakra.w300_light.clone(),
                                    font_size: 25.0,
                                    color: colors::TRUCKUI_TEXT_COLOR,
                                },
                            );
                            sample_text.style.margin = TEXT_MARGIN;

                            activity.spawn(sample_text);

                            activity.spawn(NodeBundle {
                                style: Style {
                                    justify_content: JustifyContent::FlexStart,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Percent(MARGIN_PERCENT),
                                    flex_grow: 1.0,
                                    ..default()
                                },
                                ..default()
                            });
                        });
                    // Bottom Right - 2 buttons - Exit Truck + End mission.
                    right_col
                        .spawn(NodeBundle {
                            border_color: colors::DEBUG_BCOLOR,

                            style: Style {
                                border: UiRect::all(Val::Px(1.0)),
                                padding: UiRect::all(Val::Px(1.0)),
                                margin: MARGIN,
                                min_width: Val::Px(10.0),
                                min_height: Val::Px(10.0),
                                justify_content: JustifyContent::SpaceEvenly,
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Percent(MARGIN_PERCENT),
                                column_gap: Val::Percent(MARGIN_PERCENT),
                                flex_grow: 0.01,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|buttons| {
                            buttons
                                .spawn(ButtonBundle {
                                    style: Style {
                                        min_height: Val::Px(60.0),
                                        border: MARGIN,
                                        align_content: AlignContent::Center,
                                        justify_content: JustifyContent::Center,
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::bottom(Val::Percent(MARGIN_PERCENT)),
                                        ..default()
                                    },
                                    ..default()
                                })
                                .insert(TruckButtonType::ExitTruck.into_component())
                                .with_children(|btn| {
                                    btn.spawn(TextBundle::from_section(
                                        "Exit Truck",
                                        TextStyle {
                                            font: handles.fonts.titillium.w600_semibold.clone(),
                                            font_size: 35.0,
                                            ..default()
                                        },
                                    ));
                                });
                            buttons
                                .spawn(ButtonBundle {
                                    style: Style {
                                        min_height: Val::Px(60.0),
                                        align_content: AlignContent::Center,
                                        justify_content: JustifyContent::Center,
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        border: MARGIN,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .insert(TruckButtonType::EndMission.into_component())
                                .with_children(|btn| {
                                    btn.spawn(TextBundle::from_section(
                                        "End Mission",
                                        TextStyle {
                                            font: handles.fonts.titillium.w600_semibold.clone(),
                                            font_size: 35.0,
                                            ..default()
                                        },
                                    ));
                                });
                        });
                });
        });

    // ---
}

pub fn cleanup(mut commands: Commands, qtui: Query<Entity, With<TruckUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn show_ui(mut qtui: Query<&mut Visibility, With<TruckUI>>) {
    for mut v in qtui.iter_mut() {
        *v = Visibility::Inherited;
    }
}

pub fn hide_ui(mut qtui: Query<&mut Visibility, With<TruckUI>>) {
    for mut v in qtui.iter_mut() {
        *v = Visibility::Hidden;
    }
}

pub fn keyboard(
    game_state: Res<State<root::GameState>>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if *game_state.get() != root::GameState::Truck {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        game_next_state.set(root::GameState::None);
    }
}

pub fn truckui_event_handle(
    mut ev_truckui: EventReader<TruckUIEvent>,
    mut next_state: ResMut<NextState<root::State>>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
    gg: Res<GhostGuess>,
    gc: Res<GameConfig>,
    mut q_gear: Query<(&PlayerSprite, &mut PlayerGear)>,
) {
    for ev in ev_truckui.read() {
        match ev {
            TruckUIEvent::EndMission => {
                game_next_state.set(root::GameState::None);
                next_state.set(root::State::Summary);
            }
            TruckUIEvent::ExitTruck => game_next_state.set(root::GameState::None),
            TruckUIEvent::CraftRepellent => {
                for (player, mut gear) in q_gear.iter_mut() {
                    if player.id == gc.player_id {
                        if let Some(ghost_type) = gg.ghost_type {
                            gear.craft_repellent(ghost_type);
                        }
                    }
                }
            }
        }
    }
}

fn update_sanity(
    gc: Res<GameConfig>,
    qp: Query<&PlayerSprite>,
    mut qst: Query<&mut Text, With<SanityText>>,
) {
    for player in &qp {
        if player.id != gc.player_id {
            continue;
        }
        for mut text in &mut qst {
            let new_sanity_text = format!("Player 1:\n  {:.0}% Sanity", player.sanity());
            if new_sanity_text != text.sections[0].value {
                text.sections[0].value = new_sanity_text;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (
            Ref<Interaction>,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
            &mut TruckUIButton,
        ),
        With<Button>,
    >,
    mut text_query: Query<&mut Text>,
    mut gg: ResMut<GhostGuess>,
    mut ev_truckui: EventWriter<TruckUIEvent>,
) {
    let mut selected_evidences_found = HashSet::<Evidence>::new();
    let mut selected_evidences_missing = HashSet::<Evidence>::new();
    let mut evidences_possible = HashSet::<Evidence>::new();
    let mut new_ghost_selected = None;
    for (interaction, _color, _border_color, _children, mut tui_button) in &mut interaction_query {
        if let TruckButtonType::Evidence(evidence_type) = tui_button.class {
            match tui_button.status {
                TruckButtonState::Off => {}
                TruckButtonState::Pressed => {
                    selected_evidences_found.insert(evidence_type);
                }
                TruckButtonState::Discard => {
                    selected_evidences_missing.insert(evidence_type);
                }
            }
        }

        if tui_button.disabled {
            continue;
        }
        if interaction.is_changed() && *interaction == Interaction::Pressed {
            if let Some(truckui_event) = tui_button.pressed() {
                ev_truckui.send(truckui_event);
            }
        }
        if let TruckButtonType::Ghost(ghost_type) = tui_button.class {
            if interaction.is_changed() && tui_button.status == TruckButtonState::Pressed {
                new_ghost_selected = Some(ghost_type);
            }
            if tui_button.status != TruckButtonState::Discard {
                for evidence in ghost_type.evidences() {
                    evidences_possible.insert(evidence);
                }
            }
        }
    }

    let mut ghost_selected = None;
    for (interaction, mut color, mut border_color, children, mut tui_button) in
        &mut interaction_query
    {
        let pressed = tui_button.status == TruckButtonState::Pressed;
        if let TruckButtonType::CraftRepellent = tui_button.class {
            tui_button.disabled = gg.ghost_type.is_none();
        }
        if let TruckButtonType::Evidence(ev) = tui_button.class {
            tui_button.disabled =
                !evidences_possible.contains(&ev) && tui_button.status != TruckButtonState::Discard;
        }
        if let TruckButtonType::Ghost(gh) = tui_button.class {
            let ghost_ev = gh.evidences();
            let selected_ev_count = ghost_ev.intersection(&selected_evidences_found).count();
            let missing_ev_count = ghost_ev.intersection(&selected_evidences_missing).count();
            tui_button.disabled =
                selected_ev_count < selected_evidences_found.len() || missing_ev_count > 0;

            if let Some(ghost) = new_ghost_selected.as_ref() {
                let is_this_ghost = gh == *ghost;
                if !is_this_ghost && pressed {
                    tui_button.status = TruckButtonState::Off;
                }
            }
            if tui_button.status == TruckButtonState::Pressed {
                ghost_selected = Some(gh);
            }
        }
        let interaction = if tui_button.disabled {
            // Disable mouse actions when button is disabled
            Interaction::None
        } else {
            *interaction
        };
        let mut text = text_query.get_mut(children[0]).unwrap();
        border_color.0 = tui_button.border_color(interaction);
        *color = tui_button.background_color(interaction).into();
        text.sections[0].style.color = tui_button.text_color(interaction);
    }
    gg.ghost_type = ghost_selected;
}

fn ghost_guess_system(
    mut guess_query: Query<&mut Text, With<TruckUIGhostGuess>>,
    gg: Res<GhostGuess>,
) {
    if !gg.is_changed() {
        return;
    }
    for mut text in guess_query.iter_mut() {
        text.sections[0].value = match gg.ghost_type.as_ref() {
            Some(gn) => gn.name().to_owned(),
            None => "-- Unknown --".to_string(),
        };
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(root::State::InGame), setup_ui)
        .add_systems(OnExit(root::State::InGame), cleanup)
        .add_systems(OnEnter(root::GameState::Truck), show_ui)
        .add_systems(OnExit(root::GameState::Truck), hide_ui)
        .add_event::<TruckUIEvent>()
        .init_resource::<GhostGuess>()
        .add_systems(Update, keyboard)
        .add_systems(Update, ghost_guess_system)
        .add_systems(
            FixedUpdate,
            (button_system, update_sanity).run_if(in_state(root::GameState::Truck)),
        )
        .add_systems(Update, truckui_event_handle);
}
