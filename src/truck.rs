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

pub fn setup_ui(mut commands: Commands, mut materials: ResMut<Assets<materials::UIPanelMaterial>>) {
    // Load Truck UI
    const DEBUG_BCOLOR: BorderColor = BorderColor(Color::rgba(0.0, 1.0, 1.0, 0.03));

    const TRUCKUI_BGCOLOR: Color = Color::rgba(0.082, 0.094, 0.118, 0.9);
    const TRUCKUI_PANEL_BGCOLOR: Color = Color::rgba(0.106, 0.129, 0.157, 0.9);
    // const TRUCKUI_ACCENT_COLOR: Color = Color::rgba(0.290, 0.596, 0.706, 1.0);

    const MARGIN_PERCENT: f32 = 0.5;
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
                    left_col.spawn(MaterialNodeBundle {
                        material: materials.add(UIPanelMaterial {
                            color: TRUCKUI_PANEL_BGCOLOR,
                        }),

                        style: Style {
                            border: UiRect::all(Val::Px(1.0)),
                            padding: UiRect::all(Val::Px(1.0)),
                            margin: MARGIN,
                            min_width: Val::Px(10.0),
                            min_height: Val::Px(10.0),
                            flex_grow: 1.0,
                            ..default()
                        },
                        ..default()
                    });
                    // Bottom Left - Sensors
                    left_col.spawn(MaterialNodeBundle {
                        material: materials.add(UIPanelMaterial {
                            color: TRUCKUI_PANEL_BGCOLOR,
                        }),

                        style: Style {
                            border: UiRect::all(Val::Px(1.0)),
                            padding: UiRect::all(Val::Px(1.0)),
                            margin: MARGIN,
                            min_width: Val::Px(10.0),
                            min_height: Val::Px(10.0),
                            flex_grow: 1.0,
                            ..default()
                        },
                        ..default()
                    });
                });
            // Mid content
            parent.spawn(MaterialNodeBundle {
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
                    right_col.spawn(MaterialNodeBundle {
                        material: materials.add(UIPanelMaterial {
                            color: TRUCKUI_PANEL_BGCOLOR,
                        }),

                        style: Style {
                            border: UiRect::all(Val::Px(1.0)),
                            padding: UiRect::all(Val::Px(1.0)),
                            margin: MARGIN,
                            min_width: Val::Px(10.0),
                            min_height: Val::Px(10.0),
                            flex_grow: 1.0,
                            ..default()
                        },
                        ..default()
                    });
                    // Bottom Right - 2 buttons - Exit Truck + End mission.
                    right_col.spawn(NodeBundle {
                        border_color: DEBUG_BCOLOR,

                        style: Style {
                            border: UiRect::all(Val::Px(1.0)),
                            padding: UiRect::all(Val::Px(1.0)),
                            margin: MARGIN,
                            min_width: Val::Px(10.0),
                            min_height: Val::Px(10.0),
                            flex_grow: 0.4,
                            ..default()
                        },
                        ..default()
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
