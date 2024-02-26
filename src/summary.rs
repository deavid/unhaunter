use bevy::prelude::*;

use crate::{ghosts::GhostType, root};

#[derive(Debug, Component, Clone)]
pub struct SCamera;

#[derive(Debug, Component, Clone)]
pub struct SummaryUI;

#[derive(Debug, Clone, Resource, Default)]
pub struct SummaryData {
    pub time_taken_secs: f32,
    pub ghost_types: Vec<GhostType>,
    pub repellent_used_amt: u32,
    pub ghosts_unhaunted: u32,
}

impl SummaryData {
    pub fn new(ghost_types: Vec<GhostType>) -> Self {
        Self {
            ghost_types,
            ..default()
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.init_resource::<SummaryData>()
        .add_systems(OnEnter(root::State::Summary), (setup, setup_ui))
        .add_systems(OnExit(root::State::Summary), cleanup)
        .add_systems(Update, update_time.run_if(in_state(root::State::InGame)))
        .add_systems(Update, keyboard.run_if(in_state(root::State::Summary)));
}

pub fn setup(mut commands: Commands) {
    // ui camera
    let cam = Camera2dBundle::default();
    commands.spawn(cam).insert(SCamera);
    info!("Summary camera setup");
}

pub fn cleanup(
    mut commands: Commands,
    qc: Query<Entity, With<SCamera>>,
    qu: Query<Entity, With<SummaryUI>>,
) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }
    // Despawn UI if not used
    for ui_entity in qu.iter() {
        commands.entity(ui_entity).despawn_recursive();
    }
}

pub fn update_time(
    time: Res<Time>,
    mut sd: ResMut<SummaryData>,
    game_state: Res<State<root::GameState>>,
) {
    if *game_state == root::GameState::Pause {
        return;
    }
    sd.time_taken_secs += time.delta_seconds();
}

pub fn keyboard(
    app_state: Res<State<root::State>>,
    mut app_next_state: ResMut<NextState<root::State>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if *app_state.get() != root::State::Summary {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape)
        | keyboard_input.just_pressed(KeyCode::NumpadEnter)
        | keyboard_input.just_pressed(KeyCode::Return)
    {
        app_next_state.set(root::State::MainMenu);
    }
}

pub fn setup_ui(mut commands: Commands, handles: Res<root::GameAssets>) {
    let main_color = Color::Rgba {
        red: 0.2,
        green: 0.2,
        blue: 0.2,
        alpha: 0.05,
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                //    align_self: AlignSelf::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect {
                    left: Val::Percent(10.0),
                    right: Val::Percent(10.0),
                    top: Val::Percent(5.0),
                    bottom: Val::Percent(5.0),
                },

                ..default()
            },

            ..default()
        })
        .insert(SummaryUI)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(20.0),
                        min_width: Val::Px(0.0),
                        min_height: Val::Px(64.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    },

                    ..default()
                })
                .with_children(|parent| {
                    // logo
                    parent.spawn(ImageBundle {
                        style: Style {
                            aspect_ratio: Some(130.0 / 17.0),
                            width: Val::Percent(80.0),
                            height: Val::Auto,
                            max_width: Val::Percent(80.0),
                            max_height: Val::Percent(100.0),
                            flex_shrink: 1.0,
                            ..default()
                        },
                        image: handles.images.title.clone().into(),
                        ..default()
                    });
                });
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(20.0),
                    ..default()
                },

                ..default()
            });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(60.0),
                        justify_content: JustifyContent::SpaceEvenly,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,

                        ..default()
                    },
                    background_color: main_color.into(),
                    ..default()
                })
                .with_children(|parent| {
                    // text
                    parent.spawn(TextBundle::from_section(
                        "Summary",
                        TextStyle {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 38.0,
                            color: Color::WHITE,
                        },
                    ));
                    parent.spawn(TextBundle::from_section(
                        "Time taken: 00.00.00",
                        TextStyle {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 38.0,
                            color: Color::GRAY,
                        },
                    ));
                    parent.spawn(TextBundle::from_section(
                        "Ghosts remaining: 1",
                        TextStyle {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 38.0,
                            color: Color::GRAY,
                        },
                    ));
                    parent.spawn(TextBundle::from_section(
                        "[ - Press enter to continue - ]",
                        TextStyle {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 38.0,
                            color: Color::ORANGE_RED,
                        },
                    ));
                });
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(20.0),
                    ..default()
                },

                ..default()
            });
        });
    info!("Main menu loaded");
}
