#[cfg(not(target_arch = "wasm32"))]
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use unengine_core::MenuUI;
use unfoundation_core::platform::plt::VERSION;
use unmenu_core::components::MenuItemInteractive;
use unmenu_core::events::MenuItemClicked;
use unmenu_core::mission_select::{CurrentMissionSelectMode, MissionSelectMode};
use unmenu_core::templates;
use unprofile_core::profile::PlayerProfileData;
use unsettings_core::audio::AudioSettings;
use untypes_core::states::{AppState, MapHubState};
use unui_core::assets::UiAssets;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub(crate) enum MenuID {
    Campaign,
    CustomMission,
    MultiplayerLobby,
    Hub,
    Manual,
    Settings,
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
            #[cfg(not(target_arch = "wasm32"))]
            MenuID::Quit => "Quit",
        };
        f.write_str(text)
    }
}

#[derive(Component, Debug, Default)]
pub(crate) struct MenuSound {
    despawn: bool,
}

#[derive(Component, Debug)]
pub(crate) struct MenuUILayout;

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::MainMenu), (setup, setup_ui))
        .add_systems(Update, menu_event)
        .add_systems(Update, despawn_sound)
        .add_systems(Update, manage_title_song);
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
    ui_assets: Res<UiAssets>,
    player_profile: Res<Persistent<PlayerProfileData>>,
    lobby_presence: Option<Res<untypes_core::roles::LobbyPresenceRole>>,
) {
    let mut menu_items = if lobby_presence.is_none() {
        vec![
            (MenuID::Campaign, MenuID::Campaign.to_string()),
            (MenuID::CustomMission, MenuID::CustomMission.to_string()),
            (MenuID::Hub, MenuID::Hub.to_string()),
        ]
    } else {
        vec![(
            MenuID::MultiplayerLobby,
            MenuID::MultiplayerLobby.to_string(),
        )]
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
        &ui_assets,
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
        templates::create_player_status_bar(parent, &ui_assets, &player_profile);
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

pub(crate) fn manage_title_song(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut q_sound: Query<&mut MenuSound>,
    app_state: Res<State<AppState>>,
    audio_settings: Res<Persistent<AudioSettings>>,
    global_volume: Res<bevy::audio::GlobalVolume>,
) {
    let should_play_song = !matches!(app_state.get(), AppState::InGame);

    if let Ok(mut menusound) = q_sound.single_mut() {
        if !should_play_song && !menusound.despawn {
            menusound.despawn = true;
        } else if should_play_song && menusound.despawn {
            menusound.despawn = false;
        }
    } else if should_play_song {
        // Only spawn the song if the volume is greater than 0
        let desired_volume = audio_settings.volume_music.as_f32()
            * audio_settings.volume_master.as_f32()
            * global_volume.volume.to_linear();
        if desired_volume > 0.0 {
            commands
                .spawn(MenuSound::default())
                .insert(AudioPlayer::<AudioSource>(
                    asset_server.load("music/unhaunter_intro.ogg"),
                ))
                .insert(PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Loop,
                    volume: bevy::audio::Volume::Linear(desired_volume),
                    speed: 1.0,
                    paused: false,
                    spatial: false,
                    spatial_scale: None,
                    ..default()
                });
        }
    }
}

pub(crate) fn despawn_sound(
    mut commands: Commands,
    mut qs: Query<(Entity, &mut AudioSink, &MenuSound)>,
    audio_settings: Res<Persistent<AudioSettings>>,
    global_volume: Res<bevy::audio::GlobalVolume>,
) {
    for (entity, mut sink, menusound) in &mut qs {
        let vol = sink.volume().to_linear();
        let v = if menusound.despawn {
            vol / 1.02
        } else {
            let desired_vol = audio_settings.volume_music.as_f32()
                * audio_settings.volume_master.as_f32()
                * global_volume.volume.to_linear();
            const STEPS: f32 = 120.0;
            if vol < desired_vol / 2.0 {
                f32::max(vol * 1.02, 0.002)
            } else {
                (vol * STEPS + desired_vol) / (STEPS + 1.0)
            }
        };
        sink.set_volume(bevy::audio::Volume::Linear(v));
        if v < 0.001 {
            commands.entity(entity).despawn();
            debug!("Song despawned");
        }
    }
}
