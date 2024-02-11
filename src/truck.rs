use bevy::app::App;
use bevy::prelude::*;

use crate::{
    materials::{self, UIPanelMaterial},
    root,
};

#[derive(Component, Debug)]
pub struct TruckUI;

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(root::GameState::Truck), setup_ui)
        .add_systems(OnExit(root::GameState::Truck), cleanup)
        .add_systems(Update, keyboard);
}

pub fn setup_ui(
    mut commands: Commands,
    mut materials: ResMut<Assets<materials::UIPanelMaterial>>,
    handles: Res<root::GameAssets>,
) {
    // Load Truck UI
    const DEBUG_BCOLOR: BorderColor = BorderColor(Color::rgba(0.0, 1.0, 1.0, 0.0003));

    const TRUCKUI_BGCOLOR: Color = Color::rgba(0.082, 0.094, 0.118, 0.6);
    const TRUCKUI_PANEL_BGCOLOR: Color = Color::rgba(0.106, 0.129, 0.157, 0.8);
    const TRUCKUI_ACCENT_COLOR: Color = Color::rgba(0.290, 0.596, 0.706, 1.0);
    const TRUCKUI_TEXT_COLOR: Color = Color::rgba(0.7, 0.82, 0.85, 1.0);
    const BUTTON_EXIT_TRUCK_BGCOLOR: Color = Color::rgba(0.129, 0.165, 0.122, 1.0);
    const BUTTON_EXIT_TRUCK_FGCOLOR: Color = Color::rgba(0.196, 0.275, 0.169, 1.0);
    const BUTTON_EXIT_TRUCK_TXTCOLOR: Color = Color::rgba(0.416, 0.667, 0.271, 1.0);
    const BUTTON_END_MISSION_BGCOLOR: Color = Color::rgba(0.224, 0.129, 0.122, 1.0);
    const BUTTON_END_MISSION_FGCOLOR: Color = Color::rgba(0.388, 0.200, 0.169, 1.0);
    const BUTTON_END_MISSION_TXTCOLOR: Color = Color::rgba(0.851, 0.522, 0.275, 1.0);
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
                            );

                            sanity.spawn(title);
                            // Sanity contents
                            sanity.spawn(NodeBundle {
                                border_color: TRUCKUI_ACCENT_COLOR.into(),
                                style: Style {
                                    border: UiRect::top(Val::Px(2.0)),
                                    justify_content: JustifyContent::FlexStart,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Percent(MARGIN_PERCENT),
                                    flex_grow: 0.1,
                                    ..default()
                                },
                                ..default()
                            });
                            let mut p1_sanity = TextBundle::from_section(
                                "Player 1: 90% Sanity",
                                TextStyle {
                                    font: handles.fonts.londrina.w100_thin.clone(),
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
                            );

                            sensors.spawn(title);
                            // Sensors contents
                            sensors.spawn(NodeBundle {
                                border_color: TRUCKUI_ACCENT_COLOR.into(),
                                style: Style {
                                    border: UiRect::top(Val::Px(2.0)),
                                    justify_content: JustifyContent::FlexStart,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Percent(MARGIN_PERCENT),
                                    flex_grow: 0.5,
                                    ..default()
                                },
                                ..default()
                            });
                            let mut sensor1 = TextBundle::from_section(
                                "No Sensors",
                                TextStyle {
                                    font: handles.fonts.londrina.w100_thin.clone(),
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
                    );

                    mid_blk.spawn(title);
                    // Journal contents
                    mid_blk.spawn(NodeBundle {
                        border_color: TRUCKUI_ACCENT_COLOR.into(),
                        style: Style {
                            border: UiRect::top(Val::Px(2.0)),
                            justify_content: JustifyContent::FlexStart,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Percent(MARGIN_PERCENT),
                            flex_grow: 0.2,
                            ..default()
                        },
                        ..default()
                    });
                    let mut sample_text = TextBundle::from_section(
                        "Select evidence:",
                        TextStyle {
                            font: handles.fonts.londrina.w100_thin.clone(),
                            font_size: 25.0,
                            color: TRUCKUI_TEXT_COLOR,
                        },
                    );
                    sample_text.style.margin = TEXT_MARGIN;

                    mid_blk.spawn(sample_text);

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
                            );

                            activity.spawn(title);
                            // Activity contents
                            activity.spawn(NodeBundle {
                                border_color: TRUCKUI_ACCENT_COLOR.into(),
                                style: Style {
                                    border: UiRect::top(Val::Px(2.0)),
                                    justify_content: JustifyContent::FlexStart,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Percent(MARGIN_PERCENT),
                                    flex_grow: 0.2,
                                    ..default()
                                },
                                ..default()
                            });
                            let mut sample_text = TextBundle::from_section(
                                "Instrumentation broken",
                                TextStyle {
                                    font: handles.fonts.londrina.w100_thin.clone(),
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
                                    background_color: BUTTON_EXIT_TRUCK_BGCOLOR.into(),
                                    border_color: BUTTON_EXIT_TRUCK_FGCOLOR.into(),
                                    ..default()
                                })
                                .with_children(|btn| {
                                    btn.spawn(TextBundle::from_section(
                                        "Exit Truck",
                                        TextStyle {
                                            font: handles.fonts.londrina.w400_regular.clone(),
                                            font_size: 35.0,
                                            color: BUTTON_EXIT_TRUCK_TXTCOLOR,
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
                                    background_color: BUTTON_END_MISSION_BGCOLOR.into(),
                                    border_color: BUTTON_END_MISSION_FGCOLOR.into(),
                                    ..default()
                                })
                                .with_children(|btn| {
                                    btn.spawn(TextBundle::from_section(
                                        "End Mission",
                                        TextStyle {
                                            font: handles.fonts.londrina.w400_regular.clone(),
                                            font_size: 35.0,
                                            color: BUTTON_END_MISSION_TXTCOLOR,
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
