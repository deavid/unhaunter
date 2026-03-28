#[cfg(not(target_arch = "wasm32"))]
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncommon_app_core::platform::plt::VERSION;
use uncommon_app_core::states::AppState;
use unmaphub_core::states::MapHubState;
use unmenu_core::assets::MenuAssets;
use unmenu_core::components::MenuItemInteractive;
use unmenu_core::components::MenuUI;
use unmenu_core::events::MenuItemClicked;
use unmenu_core::mission_select::{CurrentMissionSelectMode, MissionSelectMode};
use unmenu_core::templates;
use unprofile_core::profile::PlayerProfileData;

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

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::MainMenu), (setup, setup_ui))
        .add_systems(Update, menu_event);
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
    lobby_presence: Option<Res<uncommon_app_core::roles::LobbyPresenceRole>>,
    authority: Option<Res<uncommon_app_core::roles::AuthorityRole>>,
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

    debug!("Main menu created with root entity: {:?}", root_entity);
}

pub(crate) fn menu_event(
    mut click_events: MessageReader<MenuItemClicked>,
    #[cfg(not(target_arch = "wasm32"))] mut exit: MessageWriter<AppExit>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_map_hub_state: ResMut<NextState<MapHubState>>,
    mut current_mission_select_mode: ResMut<CurrentMissionSelectMode>,
    menu_items: Query<(&MenuID, &MenuItemInteractive)>,
    mut ev_disconnect: MessageWriter<uncommon_app_core::roles::DisconnectRequest>,
) {
    for ev in click_events.read() {
        if ev.state != AppState::MainMenu {
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
                    next_app_state.set(AppState::MissionSelect);
                    info!("Transitioning to MissionSelect state (for Campaign)");
                }
                MenuID::CustomMission => {
                    // For custom missions, we go to difficulty selection first
                    next_app_state.set(AppState::MapHub);
                    next_map_hub_state.set(MapHubState::DifficultySelection);
                    info!("Transitioning to MapHub/DifficultySelection state (for Custom Mission)");
                }
                MenuID::MultiplayerLobby => {
                    next_app_state.set(AppState::Lobby);
                    info!("Transitioning to Lobby state");
                }
                MenuID::Hub => {
                    next_app_state.set(AppState::Hub);
                    info!("Transitioning to Hub state");
                }
                MenuID::Manual => {
                    next_app_state.set(AppState::UserManual);
                    info!("Transitioning to UserManual state");
                }
                MenuID::Settings => {
                    next_app_state.set(AppState::SettingsMenu);
                    info!("Transitioning to SettingsMenu state");
                }
                MenuID::Disconnect => {
                    ev_disconnect.write(uncommon_app_core::roles::DisconnectRequest);
                    // The actual teardown happens in unreplicon-plugin/connection.rs.
                    // Transition back to MainMenu so setup_ui re-runs and shows the offline menu.
                    next_app_state.set(AppState::MainMenu);
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
