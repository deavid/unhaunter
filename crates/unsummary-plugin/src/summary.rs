use crate::assets::SummaryAssets;
use crate::components::{SCamera, SummaryUI, SummaryUIType};
use bevy::{color::palettes::css, prelude::*};
use unboard_core::resources::board_topology::BoardTopology;
use uncareer_core::grade::Grade;
use uncommon_app_core::platform::plt::{FONT_SCALE, UI_SCALE};
use uncommon_app_core::utils::time::format_time;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use uninvestigation_core::ghost::GhostType;
use unmission_core::summary::{ActiveMissionEvaluator, SummaryData};
use unorchestrator_core::UIContextState;
use unplayer_core::components::PlayerSprite;
use unreplicon_core::resources::LobbyPresenceRole;
use unreplicon_core::resources::LocalPlayer;
use untmxmap_core::resources::maps::Maps;
use unvitals_core::components::PlayerVitals;
use unvitals_core::events::PlayerDiedEvent;

pub(crate) fn setup(mut commands: Commands) {
    // ui camera
    commands.spawn(Camera2d).insert(SCamera);
    debug!("Summary camera setup");
}

#[derive(Resource)]
pub(crate) struct SummaryAfkTimer(pub Timer);

pub(crate) fn cleanup(
    mut commands: Commands,
    qc: Query<Entity, With<SCamera>>,
    qu: Query<Entity, With<SummaryUI>>,
) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn();
    }

    // Despawn UI if not used
    for ui_entity in qu.iter() {
        commands.entity(ui_entity).despawn();
    }
}

pub(crate) fn update_time(
    time: Res<Time>,
    mut sd: ResMut<SummaryData>,
    mut app_next_state: ResMut<NextState<UIContextState>>,
    qp: Query<(&PlayerSprite, &PlayerVitals)>,
    difficulty: Res<CurrentDifficulty>,
    mut death_timer: Local<Option<f32>>,
) {
    // TODO: Consider moving mission stat-tracking (time_taken_secs, player_count, alive_count, average_sanity)
    // and mission-end evaluation (all_dead logic) to unmission-plugin. This function currently mixes
    // summary presentation concerns with mission lifecycle concerns — ideally unmission-plugin would own
    // the end-of-mission evaluation, and unsummary-plugin would receive the snapshot via SummaryData.
    sd.difficulty = *difficulty;
    sd.time_taken_secs += time.delta_secs();
    let total_sanity: f32 = qp.iter().map(|(_, v)| v.sanity).sum();
    let player_count = qp.iter().count();
    let alive_count = qp.iter().filter(|(_, v)| v.health > 0.0).count();
    sd.player_count = player_count;
    sd.alive_count = alive_count;
    if player_count > 0 {
        sd.average_sanity = total_sanity / player_count as f32;
    }

    if player_count > 0 && alive_count == 0 {
        let now = time.elapsed_secs();
        let start = death_timer.get_or_insert(now);
        if now - *start > 1.0 {
            app_next_state.set(UIContextState::Summary);
        }
    } else {
        *death_timer = None;
    }
}

pub(crate) fn keyboard(
    app_state: Res<State<UIContextState>>,
    mut app_next_state: ResMut<NextState<UIContextState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    lobby_presence: Option<Res<LobbyPresenceRole>>,
) {
    if *app_state.get() != UIContextState::Summary {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape)
        | keyboard_input.just_pressed(KeyCode::NumpadEnter)
        | keyboard_input.just_pressed(KeyCode::Enter)
    {
        if lobby_presence.is_some() {
            app_next_state.set(UIContextState::Lobby);
        } else {
            app_next_state.set(UIContextState::MissionSelect);
        }
    }
}

pub(crate) fn insert_afk_timer(mut commands: Commands) {
    commands.insert_resource(SummaryAfkTimer(Timer::from_seconds(30.0, TimerMode::Once)));
}

pub(crate) fn remove_afk_timer(mut commands: Commands) {
    commands.remove_resource::<SummaryAfkTimer>();
}

pub(crate) fn afk_timeout(
    mut timer: ResMut<SummaryAfkTimer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    lobby_presence: Option<Res<LobbyPresenceRole>>,
    mut app_next_state: ResMut<NextState<UIContextState>>,
    time: Res<Time>,
) {
    if keyboard_input.get_just_pressed().next().is_some() {
        timer.0.reset();
        return;
    }
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        if lobby_presence.is_some() {
            app_next_state.set(UIContextState::Lobby);
        } else {
            app_next_state.set(UIContextState::MissionSelect);
        }
    }
}
pub(crate) fn setup_ui(
    mut commands: Commands,
    ui_assets: Res<SummaryAssets>,
    rsd: Res<SummaryData>,
) {
    let main_color = Color::Srgba(Srgba {
        red: 0.2,
        green: 0.2,
        blue: 0.2,
        alpha: 0.05,
    });

    // Calculate net change to bank
    let net_change = rsd.money_earned + rsd.deposit_returned_to_bank - rsd.deposit_originally_held;

    let final_bank = rsd.final_bank_total;

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
                            image: ui_assets.title.clone(),
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
                    display: Display::Grid,
                    padding: UiRect::horizontal(Val::Px(10.0 * UI_SCALE)),
                    grid_auto_rows: vec![
                        GridTrack::DEFAULT,
                        GridTrack::DEFAULT,
                        GridTrack::DEFAULT,
                    ],
                    grid_template_columns: vec![
                        GridTrack::percent(33.0),
                        GridTrack::percent(33.0),
                        GridTrack::percent(33.0),
                    ],
                    grid_template_rows: vec![
                        GridTrack::auto(),
                        GridTrack::auto(),
                        GridTrack::auto(),
                    ],

                    ..default()
                })
                .insert(BackgroundColor(main_color))
                .with_children(|parent| {
                    // Header
                    parent
                        .spawn(Text::new("Mission Summary"))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 32.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(Color::WHITE));

                    parent
                        .spawn(Text::new("Map: Unknown"))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::MapMissionName);

                    parent
                        .spawn(Text::new("Difficulty: Unknown"))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::DifficultyName);

                    // Ghost and mission details
                    parent
                        .spawn(Text::new("Ghost list"))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::GhostList);

                    parent
                        .spawn(Text::new("Time taken: 00.00.00"))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::TimeTaken);

                    parent
                        .spawn(Text::new("Players Alive: 0/0"))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::PlayersAlive);

                    parent
                        .spawn(Text::new("Ghosts unhaunted: 0/0"))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::GhostUnhaunted);

                    parent
                        .spawn(Text::new("Average Sanity: 0.0%"))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::AvgSanity);

                    parent
                        .spawn(Text::new("Repellent charges used: 0"))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::RepellentUsed);

                    // Separator
                    parent
                        .spawn(Node {
                            left: Val::Percent(1.0),
                            width: Val::Percent(98.0),
                            height: Val::Px(2.0),
                            margin: UiRect {
                                top: Val::Px(15.0),
                                bottom: Val::Px(15.0),
                                ..default()
                            },
                            grid_column: GridPlacement::span(3),
                            ..default()
                        })
                        .insert(BackgroundColor(css::GRAY.into()));

                    // Performance and financial details
                    // Grade and Score
                    parent
                        .spawn(Text::new(format!("Grade Achieved: {}", rsd.grade_achieved)))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 28.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(Node {
                            grid_column: GridPlacement::span(2),
                            ..default()
                        })
                        .insert(TextColor(Color::WHITE))
                        .insert(SummaryUIType::GradeAchieved);

                    parent
                        .spawn(Text::new(format!(
                            "Final Score: {} x {:.1} = {}",
                            rsd.base_score, rsd.difficulty_multiplier, rsd.animated_final_score
                        )))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::FinalScore);

                    // Separator
                    parent
                        .spawn(Node {
                            left: Val::Percent(1.0),
                            width: Val::Percent(98.0),
                            height: Val::Px(2.0),
                            margin: UiRect {
                                top: Val::Px(10.0),
                                bottom: Val::Px(10.0),
                                ..default()
                            },
                            grid_column: GridPlacement::span(3),
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
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 22.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::BaseReward);

                    parent
                        .spawn(Text::new(format!(
                            "Grade Multiplier: {:.1}x",
                            rsd.grade_multiplier
                        )))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 22.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::GradeMultiplier);

                    parent
                        .spawn(Text::new(format!(
                            "Calculated Earnings: ${}",
                            rsd.money_earned
                        )))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 22.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::CalculatedEarnings);

                    // Separator
                    parent
                        .spawn(Node {
                            left: Val::Percent(1.0),
                            width: Val::Percent(98.0),
                            height: Val::Px(1.0),
                            margin: UiRect {
                                top: Val::Px(5.0),
                                bottom: Val::Px(5.0),
                                ..default()
                            },
                            grid_column: GridPlacement::span(3),

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
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 22.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::InsuranceDepositHeld);

                    parent
                        .spawn(Text::new(format!(
                            "Costs/Penalties Deducted: ${}",
                            rsd.costs_deducted_from_deposit
                        )))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 22.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::CostsDeducted);

                    parent
                        .spawn(Text::new(format!(
                            "Deposit Returned to Bank: ${}",
                            rsd.deposit_returned_to_bank
                        )))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 22.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::GRAY.into()))
                        .insert(SummaryUIType::DepositReturned);

                    // Separator
                    parent
                        .spawn(Node {
                            left: Val::Percent(1.0),
                            width: Val::Percent(98.0),
                            height: Val::Px(2.0),
                            margin: UiRect {
                                top: Val::Px(10.0),
                                bottom: Val::Px(10.0),
                                ..default()
                            },
                            grid_column: GridPlacement::span(3),
                            ..default()
                        })
                        .insert(BackgroundColor(css::GRAY.into()));

                    // Final calculations
                    parent
                        .spawn(Text::new(format!("Net Change to Bank: ${}", net_change)))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 26.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(Node {
                            grid_column: GridPlacement::span(2),
                            ..default()
                        })
                        .insert(TextColor(Color::WHITE))
                        .insert(SummaryUIType::NetChange);

                    parent
                        .spawn(Text::new(format!("Final Money in Bank: ${}", final_bank)))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 26.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(Color::WHITE))
                        .insert(SummaryUIType::FinalBankTotal);

                    // Press enter prompt
                    parent.spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(5.0),
                        grid_column: GridPlacement::span(3),
                        ..default()
                    });
                    parent.spawn(Node { ..default() });

                    parent
                        .spawn(Text::new("[ - Press enter to continue - ]"))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 22.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(css::ORANGE_RED.into()));
                });
        });
    info!("Main menu loaded");
}

pub(crate) fn update_ui(
    mut qui: Query<(&SummaryUIType, &mut Text)>,
    rsd: Res<SummaryData>,
    maps: Res<Maps>,
) {
    for (sui, mut text) in &mut qui {
        match &sui {
            SummaryUIType::GhostList => {
                text.0 = format!(
                    "Ghost: {}",
                    rsd.ghost_types
                        .iter()
                        .map(|x: &GhostType| x.to_string())
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
            SummaryUIType::MapMissionName => {
                let map_name = maps
                    .maps
                    .iter()
                    .find(|m| m.path == rsd.map_path)
                    .map_or("Unknown Map", |m| &m.name);
                text.0 = format!("Map: {}", map_name);
            }
            SummaryUIType::DifficultyName => {
                text.0 = format!("Difficulty: {}", rsd.difficulty.0.difficulty_name());
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
                text.0 = format!("Final Money in Bank: ${}", rsd.final_bank_total);
            }
        }
    }
}

pub(crate) fn update_score(
    mut sd: ResMut<SummaryData>,
    app_state: Res<State<UIContextState>>,
    evaluator: Option<Res<ActiveMissionEvaluator>>,
) {
    if *app_state != UIContextState::Summary {
        return;
    }
    let Some(evaluator) = evaluator else {
        return;
    };
    let desired_score = sd.calculate_score(evaluator.0.as_ref());
    let max_delta = desired_score - sd.animated_final_score;
    let delta = (max_delta / 200).max(10).min(max_delta);
    sd.animated_final_score += delta;
}

// calculate_rewards_and_grades and finalize_profile_update have been moved to uncareer-plugin.

// Add a new system to ensure the mission ID is preserved and correctly set
pub(crate) fn store_mission_id(
    mut sd: ResMut<SummaryData>,
    board_topology: Option<Res<unboard_core::resources::board_topology::BoardTopology>>,
) {
    // Debug: Log initial state of SummaryData and BoardTopology
    info!(
        "store_mission_id: SummaryData current_mission_id='{}'",
        sd.map_path
    );

    match &board_topology {
        Some(bd) => info!(
            "store_mission_id: BoardTopology is available, map_path='{}'",
            bd.map_path
        ),
        None => info!("store_mission_id: BoardTopology is NOT available (resource not found)"),
    }

    // If the current_mission_id is empty but we have board data available, use that
    if sd.map_path.is_empty() {
        if let Some(bd) = board_topology {
            info!("Setting mission ID from board_topology: {}", bd.map_path);
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

pub(crate) fn record_death_to_summary(
    mut ev_death: MessageReader<PlayerDiedEvent>,
    mut summary_data: ResMut<SummaryData>,
    local_player: Res<LocalPlayer>,
    q_players: Query<&PlayerSprite>,
    board_topology: Res<BoardTopology>,
) {
    for ev in ev_death.read() {
        let player_uuid = q_players
            .iter()
            .find(|p| p.network_id == ev.id)
            .map(|p| p.id);

        if local_player.0 == player_uuid && player_uuid.is_some() {
            // It's us! Update summary with death-related information
            let map_path_str = board_topology.map_path.clone();

            summary_data.map_path = map_path_str;
            summary_data.deposit_originally_held = 0;
            summary_data.deposit_returned_to_bank = 0;
            summary_data.costs_deducted_from_deposit = 0;
            summary_data.money_earned = 0;
            summary_data.grade_achieved = Grade::NA;
        }
    }
}
