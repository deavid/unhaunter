use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::platform::plt::VERSION;
use uncore::resources::mission_select_mode::{CurrentMissionSelectMode, MissionSelectMode};
use uncore::states::{AppState, MapHubState};
use uncore::types::root::game_assets::GameAssets;
use uncoremenu::components::MenuItemInteractive;
use uncoremenu::systems::MenuItemClicked;
use uncoremenu::templates;
use unprofile::data::PlayerProfileData;
use unsettings::audio::AudioSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum MenuID {
    Campaign,
    CustomMission,
    Manual,
    Settings,
    Quit,
}

impl std::fmt::Display for MenuID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            MenuID::Campaign => "Campaign",
            MenuID::CustomMission => "Custom Mission",
            MenuID::Manual => "Manual",
            MenuID::Settings => "Settings",
            MenuID::Quit => "Quit",
        };
        f.write_str(text)
    }
}

#[derive(Component, Debug)]
pub struct MCamera;

#[derive(Component, Debug)]
pub struct MenuUI;

#[derive(Component, Debug, Default)]
pub struct MenuSound {
    despawn: bool,
}

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::MainMenu), (setup, setup_ui))
        .add_systems(OnExit(AppState::MainMenu), cleanup)
        .add_systems(Update, menu_event)
        .add_systems(Update, despawn_sound)
        .add_systems(Update, manage_title_song);
}

pub fn setup(mut commands: Commands, mut player_profile: ResMut<Persistent<PlayerProfileData>>) {
    commands.spawn(Camera2d).insert(MCamera);

    // Ensure player level is updated based on XP when main menu loads
    player_profile.progression.update_level();

    // Persist the updated player profile
    if let Err(e) = player_profile.persist() {
        error!("Failed to persist PlayerProfileData: {:?}", e);
    }

    info!("Main menu camera setup and player level updated");
}

pub fn setup_ui(
    mut commands: Commands,
    handles: Res<GameAssets>,
    player_profile: Res<Persistent<PlayerProfileData>>,
) {
    let menu_items = vec![
        (MenuID::Campaign, MenuID::Campaign.to_string()),
        (MenuID::CustomMission, MenuID::CustomMission.to_string()),
        (MenuID::Manual, MenuID::Manual.to_string()),
        (MenuID::Settings, MenuID::Settings.to_string()),
        #[cfg(not(target_arch = "wasm32"))]
        (MenuID::Quit, MenuID::Quit.to_string()),
    ];

    warn!("Setting up main menu with items: {:?}", menu_items);

    // Create standard menu layout using templates
    let root_entity = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(MenuUI) // Mark for cleanup
        .id();

    // Call create_standard_menu_layout directly with commands, not with parent
    let menu_layout_entity = templates::create_standard_menu_layout(
        &mut commands,
        &handles,
        &menu_items,
        0,
        Some(format!(
            "Unhaunter {}    |    [Up]/[Down]: Change    |    [Enter]: Select",
            VERSION
        )),
        MenuUI, // Pass the marker component
    );

    // Parent the menu layout to our root entity
    commands.entity(root_entity).add_child(menu_layout_entity);

    // Add the persistent player status bar as a child of root_entity
    commands.entity(root_entity).with_children(|parent| {
        templates::create_player_status_bar(parent, &handles, &player_profile);
    });

    warn!("Main menu created with root entity: {:?}", root_entity);
}

pub fn cleanup(
    mut commands: Commands,
    qc: Query<Entity, With<MCamera>>,
    qm: Query<Entity, With<MenuUI>>,
) {
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }
    for ui_entity in qm.iter() {
        commands.entity(ui_entity).despawn_recursive();
    }
}

pub fn menu_event(
    mut click_events: EventReader<MenuItemClicked>,
    mut exit: EventWriter<AppExit>,
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
                MenuID::Manual => {
                    next_app_state.set(AppState::UserManual);
                    info!("Transitioning to UserManual state");
                }
                MenuID::Settings => {
                    next_app_state.set(AppState::SettingsMenu);
                    info!("Transitioning to SettingsMenu state");
                }
                MenuID::Quit => {
                    info!("Sending AppExit event");
                    exit.send(AppExit::default());
                }
            }
        } else {
            warn!("Clicked menu item identifier {} not found in query", ev.pos);
        }
    }
}

pub fn manage_title_song(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut q_sound: Query<&mut MenuSound>,
    app_state: Res<State<AppState>>,
    audio_settings: Res<Persistent<AudioSettings>>,
) {
    let should_play_song = !matches!(app_state.get(), AppState::InGame);

    if let Ok(mut menusound) = q_sound.get_single_mut() {
        if !should_play_song && !menusound.despawn {
            menusound.despawn = true;
        } else if should_play_song && menusound.despawn {
            menusound.despawn = false;
        }
    } else if should_play_song {
        commands
            .spawn(MenuSound::default())
            .insert(AudioPlayer::<AudioSource>(
                asset_server.load("music/unhaunter_intro.ogg"),
            ))
            .insert(PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::new(
                    audio_settings.volume_music.as_f32() * audio_settings.volume_master.as_f32(),
                ),
                speed: 1.0,
                paused: false,
                spatial: false,
                spatial_scale: None,
            });
    }
}

pub fn despawn_sound(
    mut commands: Commands,
    qs: Query<(Entity, &AudioSink, &MenuSound)>,
    audio_settings: Res<Persistent<AudioSettings>>,
) {
    for (entity, sink, menusound) in &qs {
        let vol = sink.volume();
        let v = if menusound.despawn {
            vol / 1.02
        } else {
            let desired_vol =
                audio_settings.volume_music.as_f32() * audio_settings.volume_master.as_f32();
            const STEPS: f32 = 120.0;
            if vol < desired_vol / 2.0 {
                vol * 1.02
            } else {
                (vol * STEPS + desired_vol) / (STEPS + 1.0)
            }
        };
        sink.set_volume(v);
        if v < 0.001 {
            commands.entity(entity).despawn_recursive();
            info!("Song despawned");
        }
    }
}
