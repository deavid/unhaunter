use bevy::app::App;
use bevy::prelude::*;

use crate::{
    materials::{self, UIPanelMaterial},
    root,
};

#[derive(Component, Debug)]
pub struct TruckUI;

#[derive(Debug)]
pub enum TruckButtonType {
    Evidence,
    Ghost,
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
}

impl TruckUIButton {
    pub fn pressed(&mut self) {
        self.status = match self.status {
            TruckButtonState::Off => TruckButtonState::Pressed,
            TruckButtonState::Pressed => TruckButtonState::Discard,
            TruckButtonState::Discard => TruckButtonState::Off,
        }
    }
    pub fn border_color(&self, interaction: Interaction) -> Color {
        match self.class {
            TruckButtonType::Evidence | TruckButtonType::Ghost => match interaction {
                Interaction::Pressed => TRUCKUI_ACCENT3_COLOR,
                Interaction::Hovered => TRUCKUI_ACCENT_COLOR,
                Interaction::None => TRUCKUI_ACCENT2_COLOR,
            },
            TruckButtonType::ExitTruck => match interaction {
                Interaction::Pressed => BUTTON_EXIT_TRUCK_TXTCOLOR,
                Interaction::Hovered => BUTTON_EXIT_TRUCK_TXTCOLOR,
                Interaction::None => BUTTON_EXIT_TRUCK_FGCOLOR,
            },
            TruckButtonType::EndMission => match interaction {
                Interaction::Pressed => BUTTON_END_MISSION_TXTCOLOR,
                Interaction::Hovered => BUTTON_END_MISSION_TXTCOLOR,
                Interaction::None => BUTTON_END_MISSION_FGCOLOR,
            },
        }
    }
    pub fn background_color(&self, interaction: Interaction) -> Color {
        match self.class {
            TruckButtonType::Evidence | TruckButtonType::Ghost => match self.status {
                TruckButtonState::Off => TRUCKUI_BGCOLOR,
                TruckButtonState::Pressed => TRUCKUI_ACCENT2_COLOR,
                TruckButtonState::Discard => BUTTON_END_MISSION_FGCOLOR,
            },
            TruckButtonType::ExitTruck => match interaction {
                Interaction::Pressed => BUTTON_EXIT_TRUCK_FGCOLOR,
                Interaction::Hovered => BUTTON_EXIT_TRUCK_BGCOLOR,
                Interaction::None => BUTTON_EXIT_TRUCK_BGCOLOR,
            },
            TruckButtonType::EndMission => match interaction {
                Interaction::Pressed => BUTTON_END_MISSION_FGCOLOR,
                Interaction::Hovered => BUTTON_END_MISSION_BGCOLOR,
                Interaction::None => BUTTON_END_MISSION_BGCOLOR,
            },
        }
    }

    pub fn text_color(&self, _interaction: Interaction) -> Color {
        match self.class {
            TruckButtonType::Evidence | TruckButtonType::Ghost => TRUCKUI_TEXT_COLOR,
            TruckButtonType::ExitTruck => BUTTON_EXIT_TRUCK_TXTCOLOR,
            TruckButtonType::EndMission => BUTTON_END_MISSION_TXTCOLOR,
        }
    }
}

impl From<TruckButtonType> for TruckUIButton {
    fn from(value: TruckButtonType) -> Self {
        TruckUIButton {
            status: TruckButtonState::Off,
            class: value,
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(root::GameState::Truck), setup_ui)
        .add_systems(OnExit(root::GameState::Truck), cleanup)
        .add_systems(Update, keyboard)
        .add_systems(Update, button_system);
}

const DEBUG_BCOLOR: BorderColor = BorderColor(Color::rgba(0.0, 1.0, 1.0, 0.0003));

const TRUCKUI_BGCOLOR: Color = Color::rgba(0.082, 0.094, 0.118, 0.6);
const TRUCKUI_PANEL_BGCOLOR: Color = Color::rgba(0.106, 0.129, 0.157, 0.8);
const TRUCKUI_ACCENT_COLOR: Color = Color::rgba(0.290, 0.596, 0.706, 1.0);
const TRUCKUI_ACCENT2_COLOR: Color = Color::rgba(0.290, 0.596, 0.706, 0.2);
const TRUCKUI_ACCENT3_COLOR: Color = Color::rgba(0.650, 0.80, 0.856, 1.0);
const TRUCKUI_TEXT_COLOR: Color = Color::rgba(0.7, 0.82, 0.85, 1.0);
const BUTTON_EXIT_TRUCK_BGCOLOR: Color = Color::rgba(0.129, 0.165, 0.122, 1.0);
const BUTTON_EXIT_TRUCK_FGCOLOR: Color = Color::rgba(0.196, 0.275, 0.169, 1.0);
const BUTTON_EXIT_TRUCK_TXTCOLOR: Color = Color::rgba(0.416, 0.667, 0.271, 1.0);
const BUTTON_END_MISSION_BGCOLOR: Color = Color::rgba(0.224, 0.129, 0.122, 1.0);
const BUTTON_END_MISSION_FGCOLOR: Color = Color::rgba(0.388, 0.200, 0.169, 1.0);
const BUTTON_END_MISSION_TXTCOLOR: Color = Color::rgba(0.851, 0.522, 0.275, 1.0);

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
            background_color: TRUCKUI_BGCOLOR.into(),

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
            ..default()
        })
        .insert(TruckUI)
        .with_children(|parent| {
            // Left column
            parent
                .spawn(NodeBundle {
                    border_color: DEBUG_BCOLOR,
                    style: Style {
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::FlexStart,
                        flex_direction: FlexDirection::Column,
                        min_width: Val::Px(10.0),
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
                                color: TRUCKUI_PANEL_BGCOLOR,
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
                                    color: TRUCKUI_ACCENT_COLOR,
                                },
                            )
                            .with_style(Style {
                                height: Val::Px(40.0),
                                ..default()
                            });

                            sanity.spawn(title);
                            // Sanity contents
                            sanity.spawn(NodeBundle {
                                border_color: TRUCKUI_ACCENT_COLOR.into(),
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
                                    color: TRUCKUI_TEXT_COLOR,
                                },
                            );
                            p1_sanity.style.margin = TEXT_MARGIN;

                            sanity.spawn(p1_sanity);

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
                                color: TRUCKUI_PANEL_BGCOLOR,
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
                                    color: TRUCKUI_ACCENT_COLOR,
                                },
                            )
                            .with_style(Style {
                                height: Val::Px(40.0),
                                ..default()
                            });

                            sensors.spawn(title);
                            // Sensors contents
                            sensors.spawn(NodeBundle {
                                border_color: TRUCKUI_ACCENT_COLOR.into(),
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
                                    color: TRUCKUI_TEXT_COLOR,
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
                        color: TRUCKUI_PANEL_BGCOLOR,
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
                            color: TRUCKUI_ACCENT_COLOR,
                        },
                    )
                    .with_style(Style {
                        height: Val::Px(40.0),
                        ..default()
                    });

                    mid_blk.spawn(title);
                    // Journal contents
                    mid_blk.spawn(NodeBundle {
                        border_color: TRUCKUI_ACCENT_COLOR.into(),
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
                                color: TRUCKUI_TEXT_COLOR,
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
                        .with_children(|evidence| {
                            let evidences = vec![
                                "Freezing temps",
                                "Floating orbs",
                                "UV Ectoplasm",
                                "EMF Level 5",
                                "EVP Recording",
                                "Spirit Box",
                                "RL Presence",
                                "500+ cpm",
                            ];
                            for txt in evidences {
                                evidence
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
                                    .insert(TruckButtonType::Evidence.into_component())
                                    .with_children(|btn| {
                                        btn.spawn(TextBundle::from_section(
                                            txt,
                                            TextStyle {
                                                font: handles
                                                    .fonts
                                                    .titillium
                                                    .w200_extralight
                                                    .clone(),
                                                font_size: 21.0,
                                                ..default()
                                            },
                                        ));
                                    });
                            }
                        });
                    // ----
                    mid_blk.spawn(
                        TextBundle::from_section(
                            "With the above evidence we believe the ghost is:",
                            TextStyle {
                                font: handles.fonts.chakra.w300_light.clone(),
                                font_size: 25.0,
                                color: TRUCKUI_TEXT_COLOR,
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(4.0)),
                            ..default()
                        }),
                    );

                    // Ghost selection
                    mid_blk
                        .spawn(NodeBundle {
                            style: Style {
                                justify_content: JustifyContent::FlexStart,
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
                                grid_auto_rows: GridTrack::auto(),

                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|ghost_selection| {
                            let ghosts = vec![
                                "Bean Sidhe",
                                "Dullahan",
                                "Leprechaun",
                                "Barghest",
                                "Will O' Wisp",
                                "Widow",
                                "Hobs Tally",
                                "Ghoul",
                                "Afrit",
                                "Domovoi",
                                "Ghostlight",
                                "Kappa",
                                "Tengu",
                                "La Llorona",
                                "Curupira",
                                "Dybbuk",
                                "Phooka",
                                "Wisp",
                                "Gray Man",
                                "Lady in White",
                                "Maresca",
                                "Gashadokuro",
                                "Jorōgumo",
                                "Namahage",
                                "Tsuchinoko",
                                "Obayifo",
                                "Brume",
                                "Bugbear",
                                "Boggart",
                                "Grey Lady",
                                "Old Nan",
                                "Brown Lady",
                                "Morag",
                                "Fionnuala",
                                "Ailill",
                                "Cairbre",
                                "Oonagh",
                                "Mider",
                                "Orla",
                                "Finvarra",
                                "Caoilte",
                                "Ceara",
                                "Muirgheas",
                            ];
                            for txt in ghosts {
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
                                    .insert(TruckButtonType::Ghost.into_component())
                                    .with_children(|btn| {
                                        btn.spawn(TextBundle::from_section(
                                            txt,
                                            TextStyle {
                                                font: handles
                                                    .fonts
                                                    .titillium
                                                    .w200_extralight
                                                    .clone(),
                                                font_size: 21.0,
                                                ..default()
                                            },
                                        ));
                                    });
                            }
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
                    border_color: DEBUG_BCOLOR,

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
                                color: TRUCKUI_PANEL_BGCOLOR,
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
                                    color: TRUCKUI_ACCENT_COLOR,
                                },
                            )
                            .with_style(Style {
                                height: Val::Px(40.0),
                                ..default()
                            });

                            activity.spawn(title);
                            // Activity contents
                            activity.spawn(NodeBundle {
                                border_color: TRUCKUI_ACCENT_COLOR.into(),
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
                                    color: TRUCKUI_TEXT_COLOR,
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
                            border_color: DEBUG_BCOLOR,

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

pub fn keyboard(
    game_state: Res<State<root::GameState>>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if *game_state.get() != root::GameState::Truck {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        game_next_state.set(root::GameState::None);
    }
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
            &mut TruckUIButton,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, mut border_color, children, mut tui_button) in
        &mut interaction_query
    {
        let mut text = text_query.get_mut(children[0]).unwrap();
        if *interaction == Interaction::Pressed {
            tui_button.pressed();
        }

        border_color.0 = tui_button.border_color(*interaction);
        *color = tui_button.background_color(*interaction).into();
        text.sections[0].style.color = tui_button.text_color(*interaction);
    }
}
