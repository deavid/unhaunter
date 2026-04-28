#[cfg(not(target_arch = "wasm32"))]
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncommon_app_core::platform::plt::VERSION;
use uncommon_states_core::UIContextState;
use unmaphub_core::states::MapHubState;
use unmenu_core::assets::MenuAssets;
use unmenu_core::components::MenuItemInteractive;
use unmenu_core::components::MenuUI;
use unmenu_core::events::MenuItemClicked;
use unmenu_core::mission_select::{CurrentMissionSelectMode, MissionSelectMode};
use unmenu_core::templates;
use unprofile_core::profile::PlayerProfileData;
use unreplicon_core::resources::{AuthorityRole, DisconnectRequest, LobbyPresenceRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub(crate) enum MenuID {
    Campaign,
    CustomMission,
    MultiplayerLobby,
    Hub,
    Manual,
    Settings,
    Disconnect,
    #[cfg(not(target_arch = "wasm32"))]
    Quit,
}

impl std::fmt::Display for MenuID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            MenuID::Campaign => "Campaign",
            MenuID::CustomMission => "Custom Mission",
            MenuID::MultiplayerLobby => "Multiplayer Lobby",
            MenuID::Hub => "Play Online",
            MenuID::Manual => "Manual",
            MenuID::Settings => "Settings",
            MenuID::Disconnect => "Disconnect from Server",
            #[cfg(not(target_arch = "wasm32"))]
            MenuID::Quit => "Quit",
        };
        f.write_str(text)
    }
}

#[derive(Component, Debug)]
pub(crate) struct MenuUILayout;

#[derive(Component, Debug)]
pub(crate) struct UpgradeNotificationBanner;

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(UIContextState::MainMenu), (setup, setup_ui))
        .add_systems(
            Update,
            (
                menu_event,
                update_hub_button_availability,
                update_upgrade_notification,
            )
                .run_if(in_state(UIContextState::MainMenu)),
        );
}

pub(crate) fn setup(mut player_profile: ResMut<Persistent<PlayerProfileData>>) {
    // Ensure player level is updated based on XP when main menu loads
    player_profile.progression.update_level();

    // Persist the updated player profile
    if let Err(e) = player_profile.persist() {
        error!("Failed to persist PlayerProfileData: {:?}", e);
    }

    debug!("Main menu camera setup and player level updated");
}

pub(crate) fn setup_ui(
    mut commands: Commands,
    menu_assets: Res<MenuAssets>,
    player_profile: Res<Persistent<PlayerProfileData>>,
    lobby_presence: Option<Res<LobbyPresenceRole>>,
    authority: Option<Res<AuthorityRole>>,
) {
    let is_pure_client = lobby_presence.is_some() && authority.is_none();

    let mut menu_items = if is_pure_client {
        // Hub Client / Join-only: can only go to the lobby, or disconnect.
        vec![
            (
                MenuID::MultiplayerLobby,
                MenuID::MultiplayerLobby.to_string(),
            ),
            (MenuID::Disconnect, MenuID::Disconnect.to_string()),
        ]
    } else if lobby_presence.is_some() {
        // PeerHost or Dedicated with local player: show lobby entry.
        vec![(
            MenuID::MultiplayerLobby,
            MenuID::MultiplayerLobby.to_string(),
        )]
    } else {
        // Offline single-player: full menu.
        vec![
            (MenuID::Campaign, MenuID::Campaign.to_string()),
            (MenuID::CustomMission, MenuID::CustomMission.to_string()),
            (MenuID::Hub, MenuID::Hub.to_string()),
        ]
    };

    menu_items.extend(vec![
        (MenuID::Manual, MenuID::Manual.to_string()),
        (MenuID::Settings, MenuID::Settings.to_string()),
        #[cfg(not(target_arch = "wasm32"))]
        (MenuID::Quit, MenuID::Quit.to_string()),
    ]);

    debug!("Setting up main menu with items: {:?}", menu_items);

    // Create standard menu layout using templates
    let root_entity = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(MenuUI)
        .id();

    // Call create_standard_menu_layout directly with commands, not with parent
    let menu_layout_entity = templates::create_standard_menu_layout(
        &mut commands,
        &menu_assets,
        &menu_items,
        0,
        Some(format!(
            "Unhaunter {}    |    [Up]/[Down]: Change    |    [Enter]: Select",
            VERSION
        )),
        MenuUILayout,
    );

    // Parent the menu layout to our root entity
    commands.entity(root_entity).add_child(menu_layout_entity);

    // Add the persistent player status bar as a child of root_entity
    commands.entity(root_entity).with_children(|parent| {
        templates::create_player_status_bar(parent, &menu_assets, &player_profile);
    });
    // Add upgrade notification banner
    commands.entity(root_entity).with_children(|parent| {
        parent
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(20.0 * uncommon_app_core::platform::plt::UI_SCALE),
                    left: Val::Px(600.0 * uncommon_app_core::platform::plt::UI_SCALE),
                    right: Val::Px(20.0 * uncommon_app_core::platform::plt::UI_SCALE),
                    padding: UiRect::all(Val::Px(
                        10.0 * uncommon_app_core::platform::plt::UI_SCALE,
                    )),
                    ..default()
                },
                UpgradeNotificationBanner,
                Visibility::Hidden,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(""),
                    TextFont {
                        font: menu_assets.font_kode_bold.clone(),
                        font_size: 18.0 * uncommon_app_core::platform::plt::FONT_SCALE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
    });

    debug!("Main menu created with root entity: {:?}", root_entity);
}

pub(crate) fn menu_event(
    mut click_events: MessageReader<MenuItemClicked>,
    #[cfg(not(target_arch = "wasm32"))] mut exit: MessageWriter<AppExit>,
    mut next_app_state: ResMut<NextState<UIContextState>>,
    mut next_map_hub_state: ResMut<NextState<MapHubState>>,
    mut current_mission_select_mode: ResMut<CurrentMissionSelectMode>,
    menu_items: Query<(&MenuID, &MenuItemInteractive)>,
    mut ev_disconnect: MessageWriter<DisconnectRequest>,
    hub_status: Res<unhub_plugin::hub_client::HubStatus>,
) {
    for ev in click_events.read() {
        if ev.state != UIContextState::MainMenu {
            warn!("MenuItemClicked event received in state: {:?}", ev.state);
            continue;
        }
        // Find the MenuID associated with the clicked item's identifier
        if let Some((menu_id, _)) = menu_items
            .iter()
            .find(|(_, interactive)| interactive.identifier == ev.pos)
        {
            match menu_id {
                MenuID::Campaign => {
                    // Set the mission select mode to Campaign
                    current_mission_select_mode.0 = MissionSelectMode::Campaign;
                    // Transition to the unified mission selection state
                    next_app_state.set(UIContextState::MissionSelect);
                    info!("Transitioning to MissionSelect state (for Campaign)");
                }
                MenuID::CustomMission => {
                    // For custom missions, we go to difficulty selection first
                    next_app_state.set(UIContextState::MapHub);
                    next_map_hub_state.set(MapHubState::DifficultySelection);
                    info!("Transitioning to MapHub/DifficultySelection state (for Custom Mission)");
                }
                MenuID::MultiplayerLobby => {
                    next_app_state.set(UIContextState::Lobby);
                    info!("Transitioning to Lobby state");
                }
                MenuID::Hub => {
                    if hub_status.is_online {
                        next_app_state.set(UIContextState::Hub);
                        info!("Transitioning to Hub state");
                    } else {
                        warn!("Hub clicked but HubStatus is offline.");
                    }
                }
                MenuID::Manual => {
                    next_app_state.set(UIContextState::UserManual);
                    info!("Transitioning to UserManual state");
                }
                MenuID::Settings => {
                    next_app_state.set(UIContextState::SettingsMenu);
                    info!("Transitioning to SettingsMenu state");
                }
                MenuID::Disconnect => {
                    ev_disconnect.write(DisconnectRequest);
                    // The actual teardown happens in unreplicon-plugin/connection.rs.
                    // Transition back to MainMenu so setup_ui re-runs and shows the offline menu.
                    next_app_state.set(UIContextState::MainMenu);
                    info!("DisconnectRequest sent; transitioning to MainMenu");
                }
                #[cfg(not(target_arch = "wasm32"))]
                MenuID::Quit => {
                    info!("Sending AppExit event");
                    exit.write(AppExit::default());
                }
            }
        } else {
            warn!("Clicked menu item identifier {} not found in query", ev.pos);
        }
    }
}

pub(crate) fn update_upgrade_notification(
    hub_connection_status: Res<unhub_plugin::hub_client::HubConnectionStatus>,
    q_banner: Query<&Children, With<UpgradeNotificationBanner>>,
    mut q_text: Query<&mut Text>,
    mut q_text_color: Query<&mut TextColor>,
    mut q_visibility: Query<&mut Visibility, With<UpgradeNotificationBanner>>,
) {
    use unhub_client::protocol::MultiplayerStatus;

    let mut banner_visible = false;
    let mut banner_text = String::new();

    if let Some(ref status) = hub_connection_status.status {
        match status {
            MultiplayerStatus::UpdateAvailable => {
                if let Some(ref version) = hub_connection_status.upgrade_version {
                    banner_text = format!("Update available: v{}", version);
                    banner_visible = true;
                }
            }
            MultiplayerStatus::UpdateRecommended => {
                banner_text =
                    "Major update available. Update to play with more players.".to_string();
                banner_visible = true;
            }
            MultiplayerStatus::Unsupported => {
                banner_text =
                    "Version unsupported. Download the latest version to play online.".to_string();
                banner_visible = true;
            }
            MultiplayerStatus::UpToDate => {
                banner_visible = false;
            }
        }
    }

    // Update visibility
    for mut vis in q_visibility.iter_mut() {
        *vis = if banner_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Update text and color
    for children in q_banner.iter() {
        for child in children.iter() {
            if let Ok(mut text) = q_text.get_mut(child) {
                text.0 = banner_text.clone();
            }
            if let Ok(mut text_color) = q_text_color.get_mut(child) {
                text_color.0 = Color::srgb(1.0, 0.5, 0.0); // Orange
            }
        }
    }
}

pub(crate) fn update_hub_button_availability(
    hub_status: Res<unhub_plugin::hub_client::HubStatus>,
    hub_connection_status: Res<unhub_plugin::hub_client::HubConnectionStatus>,
    q_button: Query<(Entity, &MenuID, Option<&Button>, &Children), With<MenuItemInteractive>>,
    mut q_text: Query<&mut TextColor>,
    mut commands: Commands,
) {
    use unhub_client::protocol::MultiplayerStatus;

    for (entity, menu_id, button_opt, children) in q_button.iter() {
        if *menu_id == MenuID::Hub {
            let is_unsupported = hub_connection_status
                .status
                .map(|s| matches!(s, MultiplayerStatus::Unsupported))
                .unwrap_or(false);

            let should_be_enabled = hub_status.is_online && !is_unsupported;

            if should_be_enabled && button_opt.is_none() {
                // Was offline or unsupported, now online and supported -> re-enable
                commands
                    .entity(entity)
                    .insert(Button)
                    .insert(Interaction::None);

                for child in children.iter() {
                    if let Ok(mut text_color) = q_text.get_mut(child) {
                        text_color.0 = unmenu_core::colors::MENU_ITEM_COLOR_OFF;
                    }
                }
            } else if !should_be_enabled && button_opt.is_some() {
                // Was enabled, now offline or unsupported -> disable
                commands
                    .entity(entity)
                    .remove::<Button>()
                    .remove::<Interaction>();
            }

            // Continuously force the color if it is offline or unsupported so that unmenu-plugin's frame delay doesn't override it.
            if !should_be_enabled {
                let color = if is_unsupported {
                    unmenu_core::colors::MENU_ITEM_COLOR_OFF.with_alpha(0.5)
                } else {
                    unmenu_core::colors::MENU_ITEM_COLOR_OFF.with_alpha(0.3)
                };
                for child in children.iter() {
                    if let Ok(mut text_color) = q_text.get_mut(child) {
                        text_color.0 = color;
                    }
                }
            }
        }
    }
}
