use std::str::FromStr;

use bevy::prelude::*;
use bevy_persistent::Persistent;
use unassets_core::resources::maps::Maps;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unengine_core::MenuUI;
use unfoundation_core::colors;
use unfoundation_core::platform::plt::{FONT_SCALE, UI_SCALE};
use unmapload_core::events::loadlevel::LoadLevelEvent;
use unmenu_core::components::MenuMouseTracker;
use unmenu_core::events::{MenuEscapeEvent, MenuItemClicked};
use unmenu_core::templates;
use unprofile_core::profile::PlayerProfileData;
use unreplicon_core::components::SelectedMission;
use unreplicon_core::messages::RequestStartMission;
use unreplicon_core::resources::{CurrentMapSeed, LobbyData, LocalPlayer, RoomOwner};
use untypes_core::cli::CliOptions;
use untypes_core::difficulty::Difficulty;
use untypes_core::states::{AppState, LobbyScreen};
use unui_core::assets::UiAssets;

#[derive(Component)]
pub(crate) struct LobbyMainUI;

#[derive(Component)]
pub(crate) struct LobbyMapPreview;

#[derive(Component)]
pub(crate) struct LobbyMapInfo;

#[derive(Component)]
pub(crate) struct LobbyDifficultyInfo;

#[derive(Component)]
pub(crate) struct LobbyPlayerList;

#[derive(Component)]
pub(crate) struct LobbyRoomCode;

#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(crate) enum LobbyMenuAction {
    SelectMap,
    SelectDifficulty,
    StartMission,
    ExitLobby,
}

#[derive(Resource, Default)]
pub(crate) struct StateEntryTimer(pub f32);

pub(crate) fn setup_ui(
    mut commands: Commands,
    ui_assets: If<Res<UiAssets>>,
    cli: Res<CliOptions>,
    player_profile: Res<Persistent<PlayerProfileData>>,
    q_ui: Query<Entity, With<LobbyMainUI>>,
    time: Res<Time>,
    mut entry_timer: ResMut<StateEntryTimer>,
    local_player: Res<LocalPlayer>,
    room_owner: Option<Res<RoomOwner>>,
    room_ident: Option<Res<unreplicon_core::resources::RoomIdentification>>,
) {
    *entry_timer = StateEntryTimer(time.elapsed_secs());
    if !q_ui.is_empty() {
        return;
    }
    let is_room_owner = match (local_player.0, room_owner) {
        (Some(lp), Some(ro)) => lp == ro.0,
        (Some(_), None) => cli.is_authority() && !cli.is_headless(),
        _ => false,
    };

    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            MenuUI,
            LobbyMainUI,
        ))
        .id();

    commands.entity(root).with_children(|p| {
        templates::create_background(p, &ui_assets);
        templates::create_logo(p, &ui_assets);
        templates::create_player_status_bar(p, &ui_assets, &player_profile);

        // Sidebar strip for primary navigation
        let mut strip = templates::create_menu_strip::<LobbyMenuAction>(p, &ui_assets, &[], 0);
        strip.insert(MenuMouseTracker::default());

        strip.with_children(|s| {
            // Re-introducing Title without a subtitle breadcrumb
            s.spawn(Text::new("Multiplayer Lobby"))
                .insert(TextFont {
                    font: ui_assets.font_londrina_light.clone(),
                    font_size: 48.0 * FONT_SCALE,
                    ..default()
                })
                .insert(TextColor(Color::WHITE))
                .insert(Node {
                    margin: UiRect::bottom(Val::Px(24.0 * UI_SCALE)),
                    ..default()
                });

            let items = [
                (LobbyMenuAction::SelectMap, "Select Map"),
                (LobbyMenuAction::SelectDifficulty, "Select Difficulty"),
                (LobbyMenuAction::StartMission, "Start Mission"),
                (LobbyMenuAction::ExitLobby, "Exit Lobby"),
            ];

            let mut menu_idx = 0;
            for (action, label) in items {
                // Always create StartMission for everyone (visiblity controlled in update_display)
                if is_room_owner
                    || action == LobbyMenuAction::ExitLobby
                    || action == LobbyMenuAction::StartMission
                {
                    templates::create_menu_item(s, label, menu_idx, false, &ui_assets)
                        .insert(action);
                    menu_idx += 1;
                }
            }
        });

        // Right content area - Informational only (no MenuRoot here)
        let mut content = templates::create_informational_content_area(p, &ui_assets);
        content.with_children(|c| {
            // Left column: Map Preview + Details
            c.spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(60.0),
                row_gap: Val::Px(10.0 * UI_SCALE),
                ..default()
            })
            .with_children(|left| {
                left.spawn((
                    ImageNode { ..default() },
                    Node {
                        width: Val::Percent(100.0),
                        aspect_ratio: Some(16.0 / 9.0),
                        ..default()
                    },
                    LobbyMapPreview,
                ));

                left.spawn((
                    Text::new(""),
                    TextFont {
                        font: ui_assets.font_titillium_regular.clone(),
                        font_size: 18.0 * FONT_SCALE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    LobbyMapInfo,
                ));

                left.spawn((
                    Text::new(""),
                    TextFont {
                        font: ui_assets.font_titillium_light.clone(),
                        font_size: 16.0 * FONT_SCALE,
                        ..default()
                    },
                    TextColor(colors::MENU_ITEM_COLOR_ON),
                    LobbyDifficultyInfo,
                ));
            });

            // Right column: Player List
            c.spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(40.0),
                padding: UiRect::left(Val::Px(20.0 * UI_SCALE)),
                row_gap: Val::Px(8.0 * UI_SCALE),
                ..default()
            })
            .with_children(|right| {
                right.spawn((
                    Text::new("Players"),
                    TextFont {
                        font: ui_assets.font_londrina_light.clone(),
                        font_size: 32.0 * FONT_SCALE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                right.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0 * UI_SCALE),
                        ..default()
                    },
                    LobbyPlayerList,
                ));
            });
        });

        // Room Code display (Top Right)
        if let Some(ri) = room_ident.as_ref()
            && let Some(code) = ri.code.as_ref()
        {
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(40.0 * UI_SCALE),
                    top: Val::Px(40.0 * UI_SCALE),
                    ..default()
                },
                LobbyRoomCode,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(format!("Room Code: {}", code)),
                    TextFont {
                        font: ui_assets.font_kode_bold.clone(),
                        font_size: 32.0 * FONT_SCALE,
                        ..default()
                    },
                    TextColor(colors::MENU_ITEM_COLOR_ON),
                ));
            });
        }

        let help_text = if is_room_owner {
            "[ESC]: Back to Menu | [Click]: Select | [Enter]: Confirm".to_string()
        } else {
            "[ESC]: Back to Menu".to_string()
        };
        templates::create_help_text(p, &ui_assets, Some(help_text));
    });
}

pub(crate) fn cleanup_ui(mut commands: Commands, q: Query<Entity, With<LobbyMainUI>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

pub(crate) fn handle_clicks(
    mut ev_clicks: If<MessageReader<MenuItemClicked>>,
    mut ev_escape: If<MessageReader<MenuEscapeEvent>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_lobby_state: ResMut<NextState<LobbyScreen>>,
    q_actions: Query<(
        &unmenu_core::components::MenuItemInteractive,
        &LobbyMenuAction,
    )>,
    lobby_data: Res<LobbyData>,
    cli: Res<CliOptions>,
    time: Res<Time>,
    entry_timer: Res<StateEntryTimer>,
    local_player: Res<LocalPlayer>,
    room_owner: Option<Res<RoomOwner>>,
    q_selected_mission: Query<&SelectedMission>,
    mut current_map_seed: ResMut<CurrentMapSeed>,
    mut current_difficulty: ResMut<CurrentDifficulty>,
    mut ev_load: MessageWriter<LoadLevelEvent>,
    mut ev_start: MessageWriter<RequestStartMission>,
) {
    let is_room_owner = match (local_player.0, room_owner) {
        (Some(lp), Some(ro)) => lp == ro.0,
        (Some(_), None) => cli.is_authority() && !cli.is_headless(),
        _ => false,
    };

    // 0.1s guard to avoid "state bounce" from the previous screen's click event
    if time.elapsed_secs() - entry_timer.0 < 0.1 {
        ev_clicks.read().for_each(|_| {}); // Drain events
        ev_escape.read().for_each(|_| {});
        return;
    }

    if ev_escape.read().next().is_some() {
        next_app_state.set(AppState::MainMenu);
    }

    for ev in ev_clicks.read() {
        if ev.state != AppState::Lobby {
            continue;
        }

        // Find the action associated with the clicked item
        let action = q_actions
            .iter()
            .find(|(interactive, _)| interactive.identifier == ev.pos)
            .map(|(_, action)| action);

        match action {
            Some(LobbyMenuAction::SelectMap) => {
                if is_room_owner {
                    next_lobby_state.set(LobbyScreen::MapSelection);
                }
            }
            Some(LobbyMenuAction::SelectDifficulty) => {
                if is_room_owner {
                    next_lobby_state.set(LobbyScreen::DifficultySelection);
                }
            }
            Some(LobbyMenuAction::StartMission) => {
                let host_in_mission = !q_selected_mission.is_empty();

                if host_in_mission && !is_room_owner {
                    // Non-owner: join an already-running mission.
                    if let Ok(mission) = q_selected_mission.single() {
                        current_map_seed.0 = mission.map_seed;
                        if let Ok(diff) = Difficulty::from_str(&mission.difficulty_id) {
                            *current_difficulty = CurrentDifficulty::new(diff);
                        } else {
                            warn!(
                                "Unknown difficulty '{}'; keeping current",
                                mission.difficulty_id
                            );
                        }
                        ev_load.write(LoadLevelEvent {
                            map_filepath: mission.map_path.clone(),
                        });
                        info!("Non-owner joining mission: map={}", mission.map_path);
                        // AppState::InGame is set by after_level_ready when LevelReadyEvent fires.
                    }
                } else if !host_in_mission && is_room_owner {
                    // Owner: start a new mission.
                    match lobby_data.selected_map.clone() {
                        Some(map_filepath) if !map_filepath.is_empty() => {
                            let map_seed = unfoundation_core::random_seed::heavy_rng_seed();
                            info!("Room owner requesting mission start: map={}", map_filepath);
                            ev_start.write(RequestStartMission { map_seed });
                            current_map_seed.0 = map_seed;
                            if let Ok(diff) = Difficulty::from_str(&lobby_data.selected_difficulty)
                            {
                                *current_difficulty = CurrentDifficulty::new(diff);
                            } else {
                                warn!(
                                    "Unknown difficulty '{}'; keeping current",
                                    lobby_data.selected_difficulty
                                );
                            }
                            ev_load.write(LoadLevelEvent {
                                map_filepath: map_filepath.clone(),
                            });
                            // AppState::InGame is set by after_level_ready when LevelReadyEvent fires.
                        }
                        _ => {
                            warn!("Cannot start mission: no map selected");
                        }
                    }
                } else if host_in_mission && is_room_owner {
                    warn!("Owner clicked Start Mission while mission already in progress; ignored");
                }
            }
            Some(LobbyMenuAction::ExitLobby) => {
                next_app_state.set(AppState::MainMenu);
            }
            None => {}
        }
    }
}

pub(crate) fn update_display(
    lobby_data: Res<LobbyData>,
    maps: Res<Maps>,
    ui_assets: If<Res<UiAssets>>,
    asset_server: Res<AssetServer>,
    local_player: Res<LocalPlayer>,
    cli: Res<CliOptions>,
    mut q_preview: Query<&mut ImageNode, With<LobbyMapPreview>>,
    mut q_map_info: Query<&mut Text, (With<LobbyMapInfo>, Without<LobbyDifficultyInfo>)>,
    mut q_diff_info: Query<&mut Text, (With<LobbyDifficultyInfo>, Without<LobbyMapInfo>)>,
    q_player_list: Query<Entity, With<LobbyPlayerList>>,
    q_children: Query<&Children>,
    mut commands: Commands,
    mut q_menu_items: Query<(&LobbyMenuAction, &mut Visibility, &Children)>,
    mut q_text: Query<&mut Text, (Without<LobbyMapInfo>, Without<LobbyDifficultyInfo>)>,
    room_owner: Option<Res<RoomOwner>>,
    q_selected_mission: Query<Entity, With<SelectedMission>>,
) {
    let is_room_owner = match (local_player.0, room_owner) {
        (Some(lp), Some(ro)) => lp == ro.0,
        (Some(_), None) => cli.is_authority() && !cli.is_headless(),
        _ => false,
    };
    let host_in_mission = !q_selected_mission.is_empty();

    // Update Menu Items (Start/Join Mission)
    for (action, mut vis, children) in q_menu_items.iter_mut() {
        if *action == LobbyMenuAction::StartMission {
            if is_room_owner {
                *vis = Visibility::Inherited;
            } else if host_in_mission {
                *vis = Visibility::Inherited;
                for child in children {
                    if let Some(mut text) = q_text
                        .get_mut(*child)
                        .ok()
                        .filter(|t| t.as_str() != "Join Mission")
                    {
                        **text = "Join Mission".to_string();
                    }
                }
            } else {
                *vis = Visibility::Hidden;
                for child in children {
                    if let Some(mut text) = q_text
                        .get_mut(*child)
                        .ok()
                        .filter(|t| t.as_str() != "Start Mission")
                    {
                        **text = "Start Mission".to_string();
                    }
                }
            }
        } else if *action == LobbyMenuAction::SelectMap
            || *action == LobbyMenuAction::SelectDifficulty
        {
            if host_in_mission {
                *vis = Visibility::Hidden;
            } else {
                *vis = Visibility::Inherited;
            }
        }
    }

    if !lobby_data.is_changed() && !maps.is_changed() && !local_player.is_changed() {
        return;
    }

    // Update Map Preview & Info
    let map_data = lobby_data.selected_map.as_ref().and_then(|path| {
        maps.maps
            .iter()
            .find(|m| &m.path == path)
            .map(|m| &m.mission_data)
    });

    if let Ok(mut img) = q_preview.single_mut() {
        let path = map_data
            .map(|m| m.preview_image_path.clone())
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| "img/placeholder_mission.png".to_string());
        img.image = asset_server.load(path);
    }

    if let Ok(mut text) = q_map_info.single_mut() {
        if host_in_mission {
            let map_name = map_data
                .map(|m| m.display_name.as_str())
                .unwrap_or("Unknown Map");

            text.0 = format!("MISSION IN PROGRESS\n\nMap: {}", map_name,);
        } else if let Some(m) = map_data {
            text.0 = format!(
                "{}\n{}\n\n{}",
                m.display_name, m.location_name, m.flavor_text
            );
        } else {
            text.0 = "No map selected".to_string();
        }
    }

    // Update Difficulty Info
    if let Ok(mut text) = q_diff_info.single_mut() {
        if host_in_mission {
            let mut status_lines = vec!["Players:".to_string()];
            for (idx, player) in lobby_data.players.iter().enumerate() {
                let is_local = local_player.0 == Some(player.id);
                let name = if is_local {
                    "You".to_string()
                } else if idx == 0 {
                    "Host".to_string()
                } else {
                    format!("Player {}", player.id.0)
                };
                status_lines.push(format!("- {}", name));
            }
            text.0 = status_lines.join("\n");
        } else {
            let diff_str = &lobby_data.selected_difficulty;
            if let Ok(diff) = Difficulty::from_str(diff_str) {
                text.0 = format!(
                    "Difficulty: {}\n{}",
                    diff.difficulty_name(),
                    diff.difficulty_description()
                );
            } else {
                text.0 = format!("Difficulty: {}", diff_str);
            }
        }
    }

    // Update Player List (Rebuild if changed)
    if let Ok(list_entity) = q_player_list.single() {
        if let Ok(children) = q_children.get(list_entity) {
            for child in children {
                if let Ok(mut entity_cmd) = commands.get_entity(*child) {
                    entity_cmd.despawn();
                }
            }
        }
        commands.entity(list_entity).with_children(|p| {
            for (player_idx, player) in lobby_data.players.iter().enumerate() {
                let is_local = local_player.0 == Some(player.id);
                let prefix = if is_local { "\u{25BA} " } else { "" }; // ►
                let host_suffix = if player_idx == 0 { " (Host)" } else { "" };

                p.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0 * UI_SCALE),
                    ..default()
                })
                .with_children(|row| {
                    // Colored box
                    let color = colors::player_color(player.tint_color_index as usize);
                    row.spawn((
                        Node {
                            width: Val::Px(16.0 * UI_SCALE),
                            height: Val::Px(16.0 * UI_SCALE),
                            border: if is_local {
                                UiRect::all(Val::Px(2.0 * UI_SCALE))
                            } else {
                                UiRect::ZERO
                            },
                            ..default()
                        },
                        BackgroundColor(color),
                        BorderColor::all(Color::WHITE),
                    ));

                    // Player label
                    let name = player
                        .nickname
                        .clone()
                        .unwrap_or_else(|| format!("Player {}", player.id.0));
                    row.spawn((
                        Text::new(format!("{}{}{}", prefix, name, host_suffix)),
                        TextFont {
                            font: ui_assets.font_titillium_regular.clone(),
                            font_size: 20.0 * FONT_SCALE,
                            ..default()
                        },
                        TextColor(if is_local {
                            colors::MENU_ITEM_COLOR_ON
                        } else {
                            Color::WHITE
                        }),
                    ));
                });
            }
        });
    }
}
