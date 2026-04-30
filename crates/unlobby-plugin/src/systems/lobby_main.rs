use std::str::FromStr;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncommon_app_core::platform::plt::{FONT_SCALE, UI_SCALE};
use uncommon_states_core::UIContextState;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty::Difficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unlobby_core::states::LobbyScreen;
use unmapload_core::events::loadlevel::LoadLevelEvent;
use unmenu_core::assets::MenuAssets;
use unmenu_core::colors;
use unmenu_core::components::MenuMouseTracker;
use unmenu_core::components::MenuUI;
use unmenu_core::events::{MenuEscapeEvent, MenuItemClicked};
use unmenu_core::templates;
use unmission_core::types::SimulationState;
use unplayer_core::colors::player_color;
use unprofile_core::profile::PlayerProfileData;
use unrender_std::components::visuals::AlphaModulator;
use unreplicon_core::components::{LobbyInfo, SelectedMission};
use unreplicon_core::messages::{
    MissionEndReason, RequestEndMission, RequestJoinMission, RequestStartMission,
};
use unreplicon_core::resources::{AuthorityRole, LocalPlayerRole};
use unreplicon_core::resources::{CurrentMapSeed, LocalPlayer, MissionAutoJoinArmed};
use untmxmap_core::resources::maps::Maps;

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

#[derive(Component)]
pub(crate) struct DeploymentStatusText;

#[derive(Component)]
pub(crate) struct MissionLaunchControl;

#[derive(Component)]
pub(crate) struct LobbyVersionWarning;

#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(crate) enum LobbyMenuAction {
    SelectMap,
    SelectDifficulty,
    StartMission,
    AbortMission,
    ExitLobby,
}

#[derive(Resource, Default)]
pub(crate) struct StateEntryTimer(pub f32);

pub(crate) fn setup_ui(
    mut commands: Commands,
    menu_assets: Option<Res<MenuAssets>>,
    authority_role: Option<Res<AuthorityRole>>,
    local_player_role: Option<Res<LocalPlayerRole>>,
    player_profile: Option<Res<Persistent<PlayerProfileData>>>,
    q_ui: Query<Entity, With<LobbyMainUI>>,
    time: Res<Time>,
    mut entry_timer: ResMut<StateEntryTimer>,
    local_player: Res<LocalPlayer>,
    q_lobby: Query<&LobbyInfo>,
    room_ident: Option<Res<unreplicon_core::resources::RoomIdentification>>,
) {
    let Some(menu_assets) = menu_assets else {
        return;
    };
    *entry_timer = StateEntryTimer(time.elapsed_secs());
    if !q_ui.is_empty() {
        return;
    }
    let lobby_info = q_lobby.single().ok();
    let is_room_owner = match (local_player.uuid, lobby_info) {
        (lp, Some(li)) => li.leader_uuid == Some(lp),
        (_, None) => authority_role.is_some() && local_player_role.is_some(),
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
        templates::create_background(p, &menu_assets);
        templates::create_logo(p, &menu_assets);
        if let Some(profile) = player_profile {
            templates::create_player_status_bar(p, &menu_assets, &profile);
        }

        // Sidebar strip for primary navigation
        let mut strip = templates::create_menu_strip::<LobbyMenuAction>(p, &menu_assets, &[], 0);
        strip.insert(MenuMouseTracker::default());

        strip.with_children(|s| {
            // Re-introducing Title without a subtitle breadcrumb
            s.spawn(Text::new("Multiplayer Lobby"))
                .insert(TextFont {
                    font: menu_assets.font_londrina_light.clone(),
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
                (LobbyMenuAction::AbortMission, "Abort Mission"),
                (LobbyMenuAction::ExitLobby, "Exit Lobby"),
            ];

            let mut menu_idx = 0;
            for (action, label) in items {
                // Always create all menu items for everyone (visibility controlled in update_display)
                let mut menu_item =
                    templates::create_menu_item(s, label, menu_idx, false, &menu_assets);
                menu_item.insert(action);
                if action == LobbyMenuAction::StartMission {
                    menu_item.insert(MissionLaunchControl);
                }
                menu_idx += 1;
            }

            s.spawn((
                Text::new("INITIALIZING DEPLOYMENT..."),
                TextFont {
                    font: menu_assets.font_londrina_light.clone(),
                    font_size: 38.0 * FONT_SCALE,
                    ..default()
                },
                TextColor(colors::MENU_ITEM_COLOR_ON),
                Node {
                    padding: UiRect::all(Val::Px(10.0 * UI_SCALE)),
                    margin: UiRect::vertical(Val::Px(5.0 * UI_SCALE)),
                    ..default()
                },
                Visibility::Hidden,
                DeploymentStatusText,
                AlphaModulator {
                    amplitude: 0.5,
                    frequency: 3.0,
                },
            ));
        });

        // Right content area - Informational only (no MenuRoot here)
        let mut content = templates::create_informational_content_area(p, &menu_assets);
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
                        font: menu_assets.font_titillium_regular.clone(),
                        font_size: 18.0 * FONT_SCALE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    LobbyMapInfo,
                ));

                left.spawn((
                    Text::new(""),
                    TextFont {
                        font: menu_assets.font_titillium_light.clone(),
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
                        font: menu_assets.font_londrina_light.clone(),
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
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    ..default()
                },
                LobbyRoomCode,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(format!("Room Code: {}", code)),
                    TextFont {
                        font: menu_assets.font_kode_bold.clone(),
                        font_size: 32.0 * FONT_SCALE,
                        ..default()
                    },
                    TextColor(colors::MENU_ITEM_COLOR_ON),
                ));

                parent.spawn((
                    LobbyVersionWarning,
                    Text::new(""),
                    TextFont {
                        font: menu_assets.font_titillium_regular.clone(),
                        font_size: 16.0 * FONT_SCALE,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.5, 0.0)), // Orange
                ));
            });
        }

        let help_text = if is_room_owner {
            "[ESC]: Back to Menu | [Click]: Select | [Enter]: Confirm".to_string()
        } else {
            "[ESC]: Back to Menu".to_string()
        };
        templates::create_help_text(p, &menu_assets, Some(help_text));
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
    mut next_app_state: ResMut<NextState<UIContextState>>,
    mut next_lobby_state: ResMut<NextState<LobbyScreen>>,
    q_actions: Query<(
        &unmenu_core::components::MenuItemInteractive,
        &LobbyMenuAction,
    )>,
    q_lobby: Query<&LobbyInfo>,
    time: Res<Time>,
    entry_timer: Res<StateEntryTimer>,
    local_player: Res<LocalPlayer>,
    q_selected_mission: Query<&SelectedMission>,
    mut current_map_seed: ResMut<CurrentMapSeed>,
    mut current_difficulty: ResMut<CurrentDifficulty>,
    mut ev_start: MessageWriter<RequestStartMission>,
    mut ev_join: MessageWriter<RequestJoinMission>,
    mut ev_abort: MessageWriter<RequestEndMission>,
    mut ev_load: MessageWriter<LoadLevelEvent>,
) {
    let lobby_info = q_lobby.single().ok();
    let is_room_owner = match (local_player.uuid, lobby_info) {
        (lp, Some(li)) => li.leader_uuid == Some(lp),
        _ => false,
    };

    // 0.1s guard to avoid "state bounce" from the previous screen's click event
    if time.elapsed_secs() - entry_timer.0 < 0.1 {
        ev_clicks.read().for_each(|_| {}); // Drain events
        ev_escape.read().for_each(|_| {});
        return;
    }

    if ev_escape.read().next().is_some() {
        next_app_state.set(UIContextState::MainMenu);
    }

    for ev in ev_clicks.read() {
        if ev.state != UIContextState::Lobby {
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

                if host_in_mission {
                    // Everyone joins explicitly. Loading the level locally without
                    // notifying the server would leave the player without an avatar.
                    if let Ok(mission) = q_selected_mission.single() {
                        ev_join.write(RequestJoinMission);
                        current_map_seed.0 = mission.map_seed;
                        if let Ok(diff) = Difficulty::from_str(&mission.difficulty_id) {
                            info!(
                                "MULTIPLAYER_MISSION_JOIN_CLICK: map='{}' seed={} mission_difficulty={:?} previous_current_difficulty={:?}",
                                mission.map_path, mission.map_seed, diff, current_difficulty.0
                            );
                            *current_difficulty = CurrentDifficulty::new(diff);
                        } else {
                            warn!(
                                "Unknown difficulty '{}'; keeping current",
                                mission.difficulty_id
                            );
                        }
                        info!(
                            "Joining mission: map={} (join request sent)",
                            mission.map_path
                        );
                        ev_load.write(LoadLevelEvent {
                            map_filepath: mission.map_path.clone(),
                        });
                        next_app_state.set(UIContextState::MissionLoading);
                    } else {
                        warn!(
                            "MULTIPLAYER_MISSION_JOIN_CLICK: lobby UI thought a mission existed, but SelectedMission.single() failed"
                        );
                    }
                } else if is_room_owner {
                    let selected_map = lobby_info.and_then(|li| li.selected_map.clone());
                    match selected_map {
                        Some(map_filepath) if !map_filepath.is_empty() => {
                            let map_seed = uncommon_app_core::random_seed::heavy_rng_seed();
                            info!(
                                "Room owner requesting mission start: map={} seed={}",
                                map_filepath, map_seed
                            );
                            ev_start.write(RequestStartMission { map_seed });
                        }
                        _ => {
                            warn!("Cannot start mission: no map selected");
                        }
                    }
                }
            }
            Some(LobbyMenuAction::AbortMission) => {
                if is_room_owner && !q_selected_mission.is_empty() {
                    info!("Room owner requesting mission abort");
                    ev_abort.write(RequestEndMission {
                        reason: MissionEndReason::LeaderAborted,
                    });
                }
            }
            Some(LobbyMenuAction::ExitLobby) => {
                next_app_state.set(UIContextState::MainMenu);
            }
            None => {}
        }
    }
}

#[derive(SystemParam)]
pub(crate) struct LobbyUpdateParams<'w, 's> {
    pub maps: Res<'w, Maps>,
    pub asset_server: Res<'w, AssetServer>,
    pub local_player: Res<'w, LocalPlayer>,
    pub authority_role: Option<Res<'w, AuthorityRole>>,
    pub local_player_role: Option<Res<'w, LocalPlayerRole>>,
    pub q_lobby: Query<'w, 's, Ref<'static, LobbyInfo>>,
    pub q_selected_mission: Query<'w, 's, Entity, With<SelectedMission>>,
    pub auto_join_armed: Option<Res<'w, MissionAutoJoinArmed>>,
    pub hub_conn_status: Option<Res<'w, unhub_plugin::hub_client::HubConnectionStatus>>,
}

pub(crate) fn update_display(
    params: LobbyUpdateParams,
    menu_assets: Option<Res<MenuAssets>>,
    mut q_preview: Query<&mut ImageNode, With<LobbyMapPreview>>,
    mut q_map_info: Query<&mut Text, (With<LobbyMapInfo>, Without<LobbyDifficultyInfo>, Without<LobbyVersionWarning>)>,
    mut q_diff_info: Query<&mut Text, (With<LobbyDifficultyInfo>, Without<LobbyMapInfo>, Without<LobbyVersionWarning>)>,
    q_player_list: Query<Entity, With<LobbyPlayerList>>,
    q_children: Query<&Children>,
    mut commands: Commands,
    mut q_menu_items: Query<(&LobbyMenuAction, &mut Visibility, &Children)>,
    mut q_text: Query<&mut Text, (Without<LobbyMapInfo>, Without<LobbyDifficultyInfo>, Without<LobbyVersionWarning>)>,
    mut q_warning: Query<&mut Text, With<LobbyVersionWarning>>,
) {
    let Some(ui_assets) = menu_assets else {
        return;
    };

    let lobby_info = params.q_lobby.single().ok();
    let is_room_owner = match (params.local_player.uuid, lobby_info.as_deref()) {
        (lp, Some(li)) => li.leader_uuid == Some(lp),
        (_, None) => params.authority_role.is_some() && params.local_player_role.is_some(),
    };
    let host_in_mission = !params.q_selected_mission.is_empty();

    // Update Menu Items (Start/Join Mission)
    for (action, mut vis, children) in q_menu_items.iter_mut() {
        match action {
            LobbyMenuAction::StartMission => {
                let is_auto_joining =
                    host_in_mission && params.auto_join_armed.as_ref().is_some_and(|r| r.0);
                if is_auto_joining {
                    *vis = Visibility::Hidden;
                } else if host_in_mission {
                    // Server has confirmed the mission; everyone (including the owner) sees "Join Mission".
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
                } else if is_room_owner {
                    // No mission yet; owner sees "Start Mission".
                    *vis = Visibility::Inherited;
                    for child in children {
                        if let Some(mut text) = q_text
                            .get_mut(*child)
                            .ok()
                            .filter(|t| t.as_str() != "Start Mission")
                        {
                            **text = "Start Mission".to_string();
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
            }
            LobbyMenuAction::AbortMission => {
                if is_room_owner && host_in_mission {
                    *vis = Visibility::Inherited;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            LobbyMenuAction::SelectMap | LobbyMenuAction::SelectDifficulty => {
                if host_in_mission || !is_room_owner {
                    *vis = Visibility::Hidden;
                } else {
                    *vis = Visibility::Inherited;
                }
            }
            _ => {}
        }
    }

    // Update Version Warning
    if let Ok(mut warning_text) = q_warning.single_mut() {
        let warning = if let Some(hub_status) = params.hub_conn_status.as_ref() {
            match hub_status.status {
                Some(unhub_client::protocol::MultiplayerStatus::UpdateAvailable) => {
                    "Warning: Update available".to_string()
                }
                Some(unhub_client::protocol::MultiplayerStatus::UpdateRecommended) => {
                    "Warning: Update recommended for compatibility".to_string()
                }
                Some(unhub_client::protocol::MultiplayerStatus::Unsupported) => {
                    "CRITICAL: Version incompatible".to_string()
                }
                Some(unhub_client::protocol::MultiplayerStatus::Conflict) => {
                    "CRITICAL: Version conflict detected".to_string()
                }
                _ => "".to_string(),
            }
        } else {
            "".to_string()
        };
        if warning_text.0 != warning {
            warning_text.0 = warning;
        }
    }

    if lobby_info
        .as_ref()
        .map(|li| !li.is_changed())
        .unwrap_or(true)
        && !params.maps.is_changed()
        && !params.local_player.is_changed()
        && !params.hub_conn_status.is_some_and(|s| s.is_changed())
    {
        return;
    }

    // Update Map Preview & Info
    let map_data = lobby_info
        .as_deref()
        .and_then(|li| li.selected_map.as_ref())
        .and_then(|path| {
            params
                .maps
                .maps
                .iter()
                .find(|m| &m.path == path)
                .map(|m| &m.mission_data)
        });

    if let Ok(mut img) = q_preview.single_mut() {
        let path = map_data
            .map(|m| m.preview_image_path.clone())
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| "img/placeholder_mission.png".to_string());
        img.image = params.asset_server.load(path);
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
            let players = lobby_info
                .as_deref()
                .map(|li| li.players.as_slice())
                .unwrap_or_default();
            for player in players.iter() {
                let is_local = params.local_player.uuid == player.player_uuid;
                let is_leader = lobby_info
                    .as_ref()
                    .map(|li| li.leader_uuid == Some(player.player_uuid))
                    .unwrap_or(false);

                let name = if is_local {
                    "You".to_string()
                } else if is_leader {
                    "Leader".to_string()
                } else {
                    unreplicon_core::identity::generate_deterministic_name(player.player_uuid)
                };
                status_lines.push(format!("- {}", name));
            }
            text.0 = status_lines.join("\n");
        } else {
            let diff_str = lobby_info
                .as_deref()
                .map(|li| li.selected_difficulty.as_str())
                .unwrap_or("");
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
            let players = lobby_info
                .as_deref()
                .map(|li| li.players.clone())
                .unwrap_or_default();
            for player in players.iter() {
                let is_local = params.local_player.uuid == player.player_uuid;
                let prefix = if is_local { "\u{25BA} " } else { "" }; // ►
                let is_leader = lobby_info
                    .as_ref()
                    .map(|li| li.leader_uuid == Some(player.player_uuid))
                    .unwrap_or(false);
                let leader_suffix = if is_leader { " (Leader)" } else { "" };

                p.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0 * UI_SCALE),
                    ..default()
                })
                .with_children(|row| {
                    // Colored box
                    let color = player_color(player.tint_color_index as usize);
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
                    let name = player.nickname.clone().unwrap_or_else(|| {
                        unreplicon_core::identity::generate_deterministic_name(player.player_uuid)
                    });
                    row.spawn((
                        Text::new(format!("{}{}{}", prefix, name, leader_suffix)),
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

pub(crate) fn update_deployment_status_ui(
    q_mission: Query<&SelectedMission>,
    mut q_status: Query<
        (&mut Visibility, &mut Text, &mut TextColor, &AlphaModulator),
        (With<DeploymentStatusText>, Without<MissionLaunchControl>),
    >,
    auto_join_armed: Option<Res<MissionAutoJoinArmed>>,
    auto_join_delay: Option<Res<unreplicon_core::resources::MissionAutoJoinDelay>>,
    sim_state: Res<State<SimulationState>>,
    time: Res<Time>,
) {
    let should_show_status = !q_mission.is_empty() && auto_join_armed.is_some_and(|r| r.0);
    if !should_show_status {
        for (mut visibility, _, mut text_color, _) in q_status.iter_mut() {
            *visibility = Visibility::Hidden;
            text_color.0.set_alpha(1.0);
        }
        return;
    }

    let counting_down = auto_join_delay.is_some_and(|d| d.0.is_some());
    let status_text = if counting_down {
        "READY! JOINING MISSION..."
    } else if *sim_state.get() == SimulationState::Ready {
        "SYNCING TELEMETRY..."
    } else {
        "INITIALIZING DEPLOYMENT..."
    };

    for (mut visibility, mut text, mut text_color, alpha_mod) in q_status.iter_mut() {
        *visibility = Visibility::Inherited;
        if text.as_str() != status_text {
            text.0 = status_text.to_string();
        }
        let phase = (time.elapsed_secs() * alpha_mod.frequency).fract();
        let pulse = 1.0 - ((2.0 * phase) - 1.0).abs();
        // Make the pulse much more legible in UI text than world-space alpha flicker.
        let min_alpha = (1.0 - (alpha_mod.amplitude * 1.8)).clamp(0.08, 0.95);
        let alpha = min_alpha + (1.0 - min_alpha) * pulse;
        text_color.0.set_alpha(alpha.clamp(0.05, 1.0));
    }
}
