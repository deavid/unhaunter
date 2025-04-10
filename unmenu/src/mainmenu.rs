use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::platform::plt::VERSION;
use uncore::states::AppState;
use uncore::types::root::game_assets::GameAssets;
use uncoremenu::components::*;
use uncoremenu::systems::*;
use uncoremenu::templates;
use unsettings::audio::AudioSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum MenuID {
    MapHub,
    Manual,
    Settings,
    Quit,
}

impl std::fmt::Display for MenuID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            MenuID::MapHub => "New Game",
            MenuID::Settings => "Settings",
            MenuID::Manual => "Manual",
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
        .add_systems(Update, manage_title_song); // Run in all states to handle transitions
}

pub fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d).insert(MCamera);
    info!("Main menu camera setup");
}

pub fn setup_ui(mut commands: Commands, handles: Res<GameAssets>) {
    // Define menu items
    let menu_items = vec![
        (MenuID::MapHub, "New Game".to_string()),
        (MenuID::Manual, "Manual".to_string()),
        (MenuID::Settings, "Settings".to_string()),
        #[cfg(not(target_arch = "wasm32"))]
        (MenuID::Quit, "Quit".to_string()),
    ];

    warn!("Setting up main menu with items: {:?}", menu_items);

    // Create standard menu layout using templates
    let root_id = templates::create_standard_menu_layout(
        &mut commands,
        &handles,
        &menu_items,
        0, // First item selected by default
        Some(format!(
            "Unhaunter {}    |    [Up]/[Down]: Change    |    [Enter]: Select",
            VERSION
        )),
        MenuUI,
    );

    warn!("Main menu created with root entity: {:?}", root_id);
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
    menu_items: Query<(&MenuID, &MenuItemInteractive)>,
) {
    for ev in click_events.read() {
        warn!("Main menu received click event: {:?}", ev);
        warn!("Number of menu items found: {}", menu_items.iter().count());
        for (menu_id, item) in menu_items.iter() {
            warn!("Found menu item {:?} with id {}", menu_id, item.identifier);
            if item.identifier == ev.0 {
                warn!("Found matching menu item {:?}!", menu_id);
                match menu_id {
                    MenuID::MapHub => {
                        warn!("Transitioning to MapHub state");
                        next_app_state.set(AppState::MapHub);
                    }
                    MenuID::Manual => {
                        warn!("Transitioning to UserManual state");
                        next_app_state.set(AppState::UserManual);
                    }
                    MenuID::Settings => {
                        warn!("Transitioning to SettingsMenu state");
                        next_app_state.set(AppState::SettingsMenu);
                    }
                    MenuID::Quit => {
                        warn!("Sending exit event");
                        exit.send(AppExit::default());
                    }
                }
            }
        }
    }
}

// Keep the audio management functions unchanged
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
            dbg!("Song despawned");
        }
    }
}
