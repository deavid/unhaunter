use bevy::{color::palettes::css, prelude::*};
use bevy_persistent::Persistent;

use uncore::components::player_sprite::PlayerSprite;
use uncore::components::summary_ui::{SCamera, SummaryUI, SummaryUIType};
use uncore::difficulty::CurrentDifficulty;
use uncore::platform::plt::{FONT_SCALE, UI_SCALE};
use uncore::resources::maps::Maps;
use uncore::resources::summary_data::SummaryData;
use uncore::states::AppState;
use uncore::states::GameState;
use uncore::types::grade::Grade;
use uncore::types::root::game_assets::GameAssets;
use uncore::utils::time::format_time;
use unprofile::data::PlayerProfileData;

pub fn setup(mut commands: Commands) {
    // ui camera
    commands.spawn(Camera2d).insert(SCamera);
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
    game_state: Res<State<GameState>>,
    mut app_next_state: ResMut<NextState<AppState>>,
    qp: Query<&PlayerSprite>,
    difficulty: Res<CurrentDifficulty>,
) {
    if *game_state == GameState::Pause {
        return;
    }
    sd.difficulty = difficulty.clone();
    sd.time_taken_secs += time.delta_secs();
    let total_sanity: f32 = qp.iter().map(|x| x.sanity()).sum();
    let player_count = qp.iter().count();
    let alive_count = qp.iter().filter(|x| x.health > 0.0).count();
    sd.player_count = player_count;
    sd.alive_count = alive_count;
    if player_count > 0 {
        sd.average_sanity = total_sanity / player_count as f32;
    }
    if alive_count == 0 {
        app_next_state.set(AppState::Summary);
    }
}

pub fn keyboard(
    app_state: Res<State<AppState>>,
    mut app_next_state: ResMut<NextState<AppState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if *app_state.get() != AppState::Summary {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape)
        | keyboard_input.just_pressed(KeyCode::NumpadEnter)
        | keyboard_input.just_pressed(KeyCode::Enter)
    {
        app_next_state.set(AppState::MainMenu);
    }
}
pub fn setup_ui(
    mut commands: Commands,
    handles: Res<GameAssets>,
    rsd: Res<SummaryData>,
    player_profile: Res<Persistent<PlayerProfileData>>,
) {
    let main_color = Color::Srgba(Srgba {
        red: 0.2,
        green: 0.2,
        blue: 0.2,
        alpha: 0.05,
    });

    // Calculate net change to bank
    let net_change = rsd.money_earned + rsd.deposit_returned_to_bank - rsd.deposit_originally_held;

    // Calculate projected final bank total
    let final_bank = player_profile.progression.bank + net_change;

    commands
        .spawn(Node {
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
            flex_grow: 1.0,
            ..default()
        })
        .insert(BackgroundColor(main_color))
        .insert(SummaryUI)
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(20.0),
                    min_width: Val::Px(0.0),
                    min_height: Val::Px(64.0 * UI_SCALE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexStart,
                    ..default()
                })
                .with_children(|parent| {
                    // logo
                    parent
                        .spawn(ImageNode {
                            image: handles.images.title.clone(),
                            ..default()
                        })
                        .insert(Node {
                            aspect_ratio: Some(130.0 / 17.0),
                            width: Val::Percent(80.0),
                            height: Val::Auto,
                            max_width: Val::Percent(80.0),
                            max_height: Val::Percent(100.0),
                            flex_shrink: 1.0,
                            ..default()
                        });
                });
            parent.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(5.0),
                ..default()
            });
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(70.0),
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .insert(BackgroundColor(main_color))
                .with_children(|parent| {
                    // Header
                    parent
                        .spawn(Text::new("Mission Summary"))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 32.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(Color::WHITE));

                    // Ghost and mission details
                    parent
                        .spawn(Text::new("Ghost list"))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::GhostList);

                    parent
                        .spawn(Text::new("Time taken: 00.00.00"))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::TimeTaken);

                    parent
                        .spawn(Text::new("Players Alive: 0/0"))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::PlayersAlive);

                    // Separator
                    parent
                        .spawn(Node {
                            width: Val::Percent(80.0),
                            height: Val::Px(2.0),
                            margin: UiRect {
                                top: Val::Px(15.0),
                                bottom: Val::Px(15.0),
                                ..default()
                            },
                            ..default()
                        })
                        .insert(BackgroundColor(css::GRAY.into()));

                    // Performance and financial details
                    // Grade and Score
                    parent
                        .spawn(Text::new(format!("Grade Achieved: {}", rsd.grade_achieved)))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 28.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(Color::WHITE));

                    parent
                        .spawn(Text::new(format!(
                            "Final Score: {} x {:.1} = {}",
                            rsd.base_score, rsd.difficulty_multiplier, rsd.animated_final_score
                        )))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::FinalScore);

                    // Separator
                    parent
                        .spawn(Node {
                            width: Val::Percent(80.0),
                            height: Val::Px(2.0),
                            margin: UiRect {
                                top: Val::Px(10.0),
                                bottom: Val::Px(10.0),
                                ..default()
                            },
                            ..default()
                        })
                        .insert(BackgroundColor(css::GRAY.into()));

                    // Financial details
                    parent
                        .spawn(Text::new(format!(
                            "Base Mission Reward: ${}",
                            rsd.mission_reward_base
                        )))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 22.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::BaseReward);

                    parent
                        .spawn(Text::new(format!(
                            "Grade Multiplier: {:.1}x",
                            rsd.grade_multiplier
                        )))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 22.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::GradeMultiplier);

                    parent
                        .spawn(Text::new(format!(
                            "Calculated Earnings: ${}",
                            rsd.money_earned
                        )))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 22.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::CalculatedEarnings);

                    // Separator
                    parent
                        .spawn(Node {
                            width: Val::Percent(60.0),
                            height: Val::Px(1.0),
                            margin: UiRect {
                                top: Val::Px(5.0),
                                bottom: Val::Px(5.0),
                                ..default()
                            },
                            ..default()
                        })
                        .insert(BackgroundColor(css::DARK_GRAY.into()));

                    // Insurance details
                    parent
                        .spawn(Text::new(format!(
                            "Insurance Deposit Held: ${}",
                            rsd.deposit_originally_held
                        )))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 22.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(css::GRAY.into()));

                    parent
                        .spawn(Text::new(format!(
                            "Costs/Penalties Deducted: ${}",
                            rsd.costs_deducted_from_deposit
                        )))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 22.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(css::GRAY.into()));

                    parent
                        .spawn(Text::new(format!(
                            "Deposit Returned to Bank: ${}",
                            rsd.deposit_returned_to_bank
                        )))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 22.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(css::GRAY.into()));

                    // Separator
                    parent
                        .spawn(Node {
                            width: Val::Percent(80.0),
                            height: Val::Px(2.0),
                            margin: UiRect {
                                top: Val::Px(10.0),
                                bottom: Val::Px(10.0),
                                ..default()
                            },
                            ..default()
                        })
                        .insert(BackgroundColor(css::GRAY.into()));

                    // Final calculations
                    parent
                        .spawn(Text::new(format!("Net Change to Bank: ${}", net_change)))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 26.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(Color::WHITE));

                    parent
                        .spawn(Text::new(format!("Final Money in Bank: ${}", final_bank)))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 26.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(Color::WHITE));

                    // Press enter prompt
                    parent.spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(5.0),
                        ..default()
                    });

                    parent
                        .spawn(Text::new("[ - Press enter to continue - ]"))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 22.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(css::ORANGE_RED.into()));
                });
        });
    info!("Main menu loaded");
}

pub fn update_ui(
    mut qui: Query<(&SummaryUIType, &mut Text)>,
    rsd: Res<SummaryData>,
    player_profile: Res<Persistent<PlayerProfileData>>,
) {
    for (sui, mut text) in &mut qui {
        match &sui {
            SummaryUIType::GhostList => {
                text.0 = format!(
                    "Ghost: {}",
                    rsd.ghost_types
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            SummaryUIType::TimeTaken => {
                text.0 = format!("Time taken: {}", format_time(rsd.time_taken_secs))
            }
            SummaryUIType::AvgSanity => {
                text.0 = format!("Average Sanity: {:.1}%", rsd.average_sanity)
            }
            SummaryUIType::GhostUnhaunted => {
                text.0 = format!(
                    "Ghosts unhaunted: {}/{}",
                    rsd.ghosts_unhaunted,
                    rsd.ghost_types.len()
                )
            }
            SummaryUIType::PlayersAlive => {
                text.0 = format!("Players Alive: {}/{}", rsd.alive_count, rsd.player_count)
            }
            SummaryUIType::RepellentUsed => {
                text.0 = format!("Repellent charges used: {}", rsd.repellent_used_amt)
            }
            SummaryUIType::FinalScore => {
                // Format the score calculation using the stored base_score and difficulty_multiplier
                text.0 = format!(
                    "Final Score: {} x {:.1} = {}",
                    rsd.base_score, rsd.difficulty_multiplier, rsd.animated_final_score
                );
            }
            SummaryUIType::GradeAchieved => {
                text.0 = format!("Grade Achieved: {}", rsd.grade_achieved);
            }
            SummaryUIType::BaseReward => {
                text.0 = format!("Base Mission Reward: ${}", rsd.mission_reward_base);
            }
            SummaryUIType::GradeMultiplier => {
                // Use the stored grade multiplier instead of recalculating
                text.0 = format!("Grade Multiplier: {:.1}x", rsd.grade_multiplier);
            }
            SummaryUIType::CalculatedEarnings => {
                text.0 = format!("Calculated Earnings: ${}", rsd.money_earned);
            }
            SummaryUIType::InsuranceDepositHeld => {
                text.0 = format!("Insurance Deposit Held: ${}", rsd.deposit_originally_held);
            }
            SummaryUIType::CostsDeducted => {
                text.0 = format!(
                    "Costs/Penalties Deducted: ${}",
                    rsd.costs_deducted_from_deposit
                );
            }
            SummaryUIType::DepositReturned => {
                text.0 = format!(
                    "Deposit Returned to Bank: ${}",
                    rsd.deposit_returned_to_bank
                );
            }
            SummaryUIType::NetChange => {
                let net_change =
                    rsd.money_earned + rsd.deposit_returned_to_bank - rsd.deposit_originally_held;
                text.0 = format!("Net Change to Bank: ${}", net_change);
            }
            SummaryUIType::FinalBankTotal => {
                let net_change =
                    rsd.money_earned + rsd.deposit_returned_to_bank - rsd.deposit_originally_held;
                let final_bank = player_profile.progression.bank + net_change;
                text.0 = format!("Final Bank Total: ${}", final_bank);
            }
        }
    }
}

pub fn update_score(mut sd: ResMut<SummaryData>, app_state: Res<State<AppState>>) {
    if *app_state != AppState::Summary {
        return;
    }
    let desired_score = sd.calculate_score();
    let max_delta = desired_score - sd.animated_final_score;
    let delta = (max_delta / 200).max(10).min(max_delta);
    sd.animated_final_score += delta;
}

pub fn calculate_rewards_and_grades(
    mut sd: ResMut<SummaryData>,
    maps: Res<Maps>,
    app_state: Res<State<AppState>>,
) {
    if *app_state != AppState::Summary {
        return;
    }

    // Debug: Log current state of SummaryData
    info!(
        "Calculating rewards and grades. Initial SummaryData: {:?}",
        *sd
    );

    // Ensure we have calculated the base score before proceeding
    if sd.base_score == 0 && sd.mission_successful {
        // Only calculate if not already done and mission was potentially successful
        sd.calculate_score();
        info!(
            "Calculated score before grading. New base_score: {}",
            sd.base_score
        );
    }

    // Initialize grade and base reward assuming failure or N/A case first
    // sd.mission_reward_base is defaulted to 0 from SummaryData, which is fine for these cases.
    sd.grade_achieved = Grade::NA;

    if sd.mission_successful {
        if let Some(map) = maps.maps.iter().find(|map| map.path == sd.map_path) {
            // Use mission_data from the map instead of TmxMap properties directly
            let mission_data = &map.mission_data;
            let base_score = sd.base_score;

            // Determine grade for successful mission using mission data
            sd.grade_achieved = Grade::from_score(
                base_score,
                mission_data.grade_a_score_threshold,
                mission_data.grade_b_score_threshold,
                mission_data.grade_c_score_threshold,
                mission_data.grade_d_score_threshold,
            );

            // Set base reward only if mission was successful and mission data found
            sd.mission_reward_base = mission_data.mission_reward_base;

            info!(
                "Mission successful path: Base score {}, Determined grade {}. Base reward ${}",
                sd.base_score, sd.grade_achieved, sd.mission_reward_base
            );
        } else {
            warn!(
                "Map not found for mission ID: {}. Grade remains NA.",
                sd.map_path
            );
        }
    } else {
        info!(
            "Mission not successful. Grade remains NA. Base score: {}",
            sd.base_score
        );
    }

    // Consistently set grade_multiplier from the determined grade_achieved
    sd.grade_multiplier = sd.grade_achieved.multiplier();

    // Calculate money_earned based on the final grade, mission success, and base reward
    if sd.mission_successful && sd.grade_achieved != Grade::NA {
        // Only earn money if mission was successful AND a valid grade (not NA) was achieved
        // (which implies map data and mission data were found, and mission_reward_base was set)
        sd.money_earned = (sd.mission_reward_base as f64 * sd.grade_multiplier)
            .round()
            .max(0.0) as i64;
    } else {
        sd.money_earned = 0; // No earnings if mission failed or grade is NA (multiplier would be 0)
    }

    info!(
        "Finalized grade: {}, multiplier: {:.1}, money_earned: ${}, base_reward_used: ${}",
        sd.grade_achieved, sd.grade_multiplier, sd.money_earned, sd.mission_reward_base
    );
}

pub fn finalize_profile_update(
    sd: Res<SummaryData>,
    mut player_profile: ResMut<Persistent<PlayerProfileData>>,
    app_state: Res<State<AppState>>,
    maps: Res<Maps>,
) {
    if *app_state != AppState::Summary {
        return;
    }

    if sd.money_earned > 0 {
        player_profile.progression.bank += sd.money_earned;
    }

    // Get the mission data to find the difficulty enum
    let difficulty = if let Some(map) = maps.maps.iter().find(|map| map.path == sd.map_path) {
        // Use the mission_data's difficulty as the key for map statistics
        map.mission_data.difficulty
    } else {
        // If we can't find the map, use the difficulty enum directly from SummaryData's CurrentDifficulty
        warn!(
            "Map not found for mission ID: {}. Using current difficulty from summary data.",
            sd.map_path
        );
        sd.difficulty.0.difficulty
    };

    // Update map statistics for this particular map and difficulty
    let map_path = sd.map_path.clone();
    let map_stats = player_profile
        .map_statistics
        .entry(map_path)
        .or_default()
        .entry(difficulty)
        .or_default();

    // Update mission completion stats
    map_stats.total_play_time_seconds += sd.time_taken_secs as f64;

    // If mission was successful, update completion time
    if sd.mission_successful {
        map_stats.total_missions_completed += 1;
        map_stats.total_mission_completed_time_seconds += sd.time_taken_secs as f64;
    }

    // Update best score and grade
    map_stats.best_score = map_stats.best_score.max(sd.full_score);
    map_stats.best_grade = map_stats.best_grade.max(sd.grade_achieved);

    if sd.mission_successful {
        // Update global statistics
        player_profile.statistics.total_missions_completed += 1;
    }

    player_profile.statistics.total_play_time_seconds += sd.time_taken_secs as f64;

    // Add final score to player XP
    player_profile.progression.player_xp += sd.full_score;

    // Update player level
    player_profile.progression.update_level();

    if let Err(e) = player_profile.persist() {
        error!("Failed to persist player profile: {:?}", e);
    }
}

pub struct UnhaunterSummaryPlugin;

impl Plugin for UnhaunterSummaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SummaryData>()
            .add_systems(
                OnEnter(AppState::Summary),
                (
                    setup,
                    store_mission_id,
                    calculate_rewards_and_grades,
                    setup_ui,
                    finalize_profile_update,
                )
                    .chain(),
            )
            .add_systems(OnExit(AppState::Summary), cleanup)
            .add_systems(FixedUpdate, update_time.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                (keyboard, update_ui, update_score).run_if(in_state(AppState::Summary)),
            );
    }
}

// Add a new system to ensure the mission ID is preserved and correctly set
pub fn store_mission_id(
    mut sd: ResMut<SummaryData>,
    board_data: Option<Res<uncore::resources::board_data::BoardData>>,
) {
    // Debug: Log initial state of SummaryData and BoardData
    info!(
        "store_mission_id: SummaryData current_mission_id='{}'",
        sd.map_path
    );

    match &board_data {
        Some(bd) => info!(
            "store_mission_id: BoardData is available, map_path='{}'",
            bd.map_path
        ),
        None => info!("store_mission_id: BoardData is NOT available (resource not found)"),
    }

    // If the current_mission_id is empty but we have board data available, use that
    if sd.map_path.is_empty() {
        if let Some(bd) = board_data {
            info!("Setting mission ID from board_data: {}", bd.map_path);
            sd.map_path = bd.map_path.clone();
        } else {
            warn!("No board data available to set mission ID");

            // For debugging - examine SummaryData to see what ghost types exist
            info!(
                "Ghost types in SummaryData: {:?}, unhaunted: {}",
                sd.ghost_types, sd.ghosts_unhaunted
            );
        }
    } else {
        info!("Using existing mission ID: {}", sd.map_path);
    }
}
