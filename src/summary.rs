use bevy::prelude::*;

use crate::{ghost_definitions::GhostType, player::PlayerSprite, root, utils};

#[derive(Debug, Component, Clone)]
pub struct SCamera;

#[derive(Debug, Component, Clone)]
pub struct SummaryUI;

#[derive(Debug, Component, Clone)]
pub enum SummaryUIType {
    GhostList,
    TimeTaken,
    GhostUnhaunted,
    RepellentUsed,
    AvgSanity,
    PlayersAlive,
    FinalScore,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum Difficulty {
    Training,
    Beginner,
    Easy,
    #[default]
    Normal,
    Medium,
    Hard,
    Expert,
    Master,
    Insane,
}

impl Difficulty {
    fn get_multiplier(&self) -> f64 {
        match self {
            Difficulty::Training => 1.0,
            Difficulty::Beginner => 1.5,
            Difficulty::Easy => 2.8,
            Difficulty::Normal => 4.0,
            Difficulty::Medium => 5.5,
            Difficulty::Hard => 7.8,
            Difficulty::Expert => 11.0,
            Difficulty::Master => 15.5,
            Difficulty::Insane => 22.0,
        }
    }
}

#[derive(Debug, Clone, Resource, Default)]
pub struct SummaryData {
    pub time_taken_secs: f32,
    pub ghost_types: Vec<GhostType>,
    pub repellent_used_amt: u32,
    pub ghosts_unhaunted: u32,
    pub final_score: i64,
    pub difficulty: Difficulty,
    pub average_sanity: f32,
    pub player_count: usize,
    pub alive_count: usize,
}

impl SummaryData {
    pub fn new(ghost_types: Vec<GhostType>) -> Self {
        Self {
            ghost_types,
            difficulty: Difficulty::Insane,
            ..default()
        }
    }
    pub fn calculate_score(&self) -> i64 {
        let mut score = (250.0 * self.ghosts_unhaunted as f64)
            / (1.0 + self.repellent_used_amt as f64)
            / (1.0 + (self.ghost_types.len() as u32 - self.ghosts_unhaunted) as f64);

        // Sanity modifier
        score *= (self.average_sanity as f64 + 30.0) / 50.0;

        // Apply difficulty multiplier
        score *= self.difficulty.get_multiplier();

        if self.player_count == self.alive_count {
            // Apply time bonus multiplier
            score *= 1.0 + 360.0 / (60.0 + self.time_taken_secs as f64);
        } else {
            score *= self.alive_count as f64 / (self.player_count as f64 + 1.0);
        }

        // Ensure score is within a reasonable range
        score.clamp(0.0, 1000000.0) as i64
    }
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
    mut app_next_state: ResMut<NextState<root::State>>,
    qp: Query<&PlayerSprite>,
) {
    if *game_state == root::GameState::Pause {
        return;
    }
    sd.time_taken_secs += time.delta_seconds();

    let total_sanity: f32 = qp.iter().map(|x| x.sanity()).sum();
    let player_count = qp.iter().count();
    let alive_count = qp.iter().filter(|x| x.health > 0.0).count();
    sd.player_count = player_count;
    sd.alive_count = alive_count;
    if player_count > 0 {
        sd.average_sanity = total_sanity / player_count as f32;
    }
    if alive_count == 0 {
        app_next_state.set(root::State::Summary);
    }
}

pub fn keyboard(
    app_state: Res<State<root::State>>,
    mut app_next_state: ResMut<NextState<root::State>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if *app_state.get() != root::State::Summary {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape)
        | keyboard_input.just_pressed(KeyCode::NumpadEnter)
        | keyboard_input.just_pressed(KeyCode::Enter)
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
                    parent
                        .spawn(TextBundle::from_section(
                            "Ghost list",
                            TextStyle {
                                font: handles.fonts.londrina.w300_light.clone(),
                                font_size: 38.0,
                                color: Color::GRAY,
                            },
                        ))
                        .insert(SummaryUIType::GhostList);
                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(10.0),
                            ..default()
                        },

                        ..default()
                    });

                    parent
                        .spawn(TextBundle::from_section(
                            "Time taken: 00.00.00",
                            TextStyle {
                                font: handles.fonts.londrina.w300_light.clone(),
                                font_size: 38.0,
                                color: Color::GRAY,
                            },
                        ))
                        .insert(SummaryUIType::TimeTaken);
                    parent
                        .spawn(TextBundle::from_section(
                            "Average Sanity: 00",
                            TextStyle {
                                font: handles.fonts.londrina.w300_light.clone(),
                                font_size: 38.0,
                                color: Color::GRAY,
                            },
                        ))
                        .insert(SummaryUIType::AvgSanity);
                    parent
                        .spawn(TextBundle::from_section(
                            "Ghosts unhaunted: 0/1",
                            TextStyle {
                                font: handles.fonts.londrina.w300_light.clone(),
                                font_size: 38.0,
                                color: Color::GRAY,
                            },
                        ))
                        .insert(SummaryUIType::GhostUnhaunted);
                    parent
                        .spawn(TextBundle::from_section(
                            "Repellent charges used: 0",
                            TextStyle {
                                font: handles.fonts.londrina.w300_light.clone(),
                                font_size: 38.0,
                                color: Color::GRAY,
                            },
                        ))
                        .insert(SummaryUIType::RepellentUsed);
                    parent
                        .spawn(TextBundle::from_section(
                            "Players Alive: 0/0",
                            TextStyle {
                                font: handles.fonts.londrina.w300_light.clone(),
                                font_size: 38.0,
                                color: Color::GRAY,
                            },
                        ))
                        .insert(SummaryUIType::PlayersAlive);

                    parent
                        .spawn(TextBundle::from_section(
                            "Final Score: 0",
                            TextStyle {
                                font: handles.fonts.londrina.w300_light.clone(),
                                font_size: 38.0,
                                color: Color::GRAY,
                            },
                        ))
                        .insert(SummaryUIType::FinalScore);

                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(20.0),
                            ..default()
                        },

                        ..default()
                    });
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

pub fn update_ui(mut qui: Query<(&SummaryUIType, &mut Text)>, rsd: Res<SummaryData>) {
    for (sui, mut text) in &mut qui {
        match &sui {
            SummaryUIType::GhostList => {
                text.sections[0].value = format!(
                    "Ghost: {}",
                    rsd.ghost_types
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }

            SummaryUIType::TimeTaken => {
                text.sections[0].value =
                    format!("Time taken: {}", utils::format_time(rsd.time_taken_secs))
            }
            SummaryUIType::AvgSanity => {
                text.sections[0].value = format!("Average Sanity: {:.1}%", rsd.average_sanity)
            }
            SummaryUIType::GhostUnhaunted => {
                text.sections[0].value = format!(
                    "Ghosts unhaunted: {}/{}",
                    rsd.ghosts_unhaunted,
                    rsd.ghost_types.len()
                )
            }
            SummaryUIType::PlayersAlive => {
                text.sections[0].value =
                    format!("Players Alive: {}/{}", rsd.alive_count, rsd.player_count)
            }
            SummaryUIType::RepellentUsed => {
                text.sections[0].value =
                    format!("Repellent charges used: {}", rsd.repellent_used_amt)
            }
            SummaryUIType::FinalScore => {
                text.sections[0].value = format!("Final Score: {}", rsd.final_score)
            }
        }
    }
}

pub fn update_score(mut sd: ResMut<SummaryData>, app_state: Res<State<root::State>>) {
    if *app_state != root::State::Summary {
        return;
    }
    let desired_score = sd.calculate_score();
    let max_delta = desired_score - sd.final_score;
    let delta = (max_delta / 200).max(10).min(max_delta);
    sd.final_score += delta;
}

pub fn app_setup(app: &mut App) {
    app.init_resource::<SummaryData>()
        .add_systems(OnEnter(root::State::Summary), (setup, setup_ui))
        .add_systems(OnExit(root::State::Summary), cleanup)
        .add_systems(
            FixedUpdate,
            update_time.run_if(in_state(root::State::InGame)),
        )
        .add_systems(
            Update,
            (keyboard, update_ui, update_score).run_if(in_state(root::State::Summary)),
        );
}
