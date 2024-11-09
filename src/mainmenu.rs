use crate::platform;
use crate::platform::plt::IS_WASM;
use crate::platform::plt::UI_SCALE;
use crate::root;
use bevy::app::AppExit;
use bevy::color::palettes::css;
use bevy::prelude::*;

const MENU_ITEM_COLOR_OFF: Color = Color::Srgba(css::GRAY);
const MENU_ITEM_COLOR_ON: Color = Color::Srgba(css::ORANGE_RED);

// Usual value is 0.2
const MUSIC_VOLUME: f32 = 0.00002;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuID {
    MapHub,
    _Options,
    Manual,
    Quit,
}

#[derive(Debug, Copy, Clone, Event)]
pub struct MenuEvent(pub MenuID);

#[derive(Component)]
pub struct Menu {
    pub selected: MenuID,
}

impl Menu {
    pub fn items() -> &'static [MenuID] {
        if IS_WASM {
            &[
                MenuID::MapHub,
                MenuID::Manual,
                // MenuID::Options,
            ]
        } else {
            &[
                MenuID::MapHub,
                MenuID::Manual,
                // MenuID::Options,
                MenuID::Quit,
            ]
        }
    }

    pub fn item_idx(&self) -> i64 {
        for (n, item) in Menu::items().iter().enumerate() {
            if item == &self.selected {
                return n as i64;
            }
        }

        // We return zero for error which is the first item.
        error!("invalid item for item_idx - first item is assumed");
        0
    }

    pub fn idx_to_item(idx: i64) -> MenuID {
        let idx = idx.rem_euclid(Menu::items().len() as i64);
        Menu::items()[idx as usize]
    }

    pub fn next_item(&mut self) {
        self.selected = Menu::idx_to_item(self.item_idx() + 1);
    }

    pub fn previous_item(&mut self) {
        self.selected = Menu::idx_to_item(self.item_idx() - 1);
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            selected: MenuID::MapHub,
        }
    }
}

#[derive(Component, Debug)]
pub struct MenuItem {
    identifier: MenuID,
    highlighted: bool,
}

impl MenuItem {
    pub fn new(identifier: MenuID) -> Self {
        MenuItem {
            identifier,
            highlighted: false,
        }
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
    app.add_systems(Update, keyboard)
        .add_systems(Update, item_logic)
        .add_systems(Update, menu_event)
        .add_event::<MenuEvent>()
        .add_systems(Update, despawn_sound)
        .add_systems(OnEnter(root::State::MainMenu), (setup, setup_ui))
        .add_systems(OnExit(root::State::MainMenu), cleanup)
        .add_systems(
            Update,
            manage_title_song.run_if(state_changed::<root::State>),
        );
}

pub fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // ui camera
    let cam = Camera2dBundle::default();
    commands.spawn(cam).insert(MCamera);
    info!("Main menu camera setup");
}

pub fn cleanup(
    mut commands: Commands,
    qc: Query<Entity, With<MCamera>>,
    qm: Query<Entity, With<MenuUI>>,
) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }

    // Despawn menu UI if not used
    for ui_entity in qm.iter() {
        commands.entity(ui_entity).despawn_recursive();
    }
}

pub fn despawn_sound(mut commands: Commands, qs: Query<(Entity, &AudioSink, &MenuSound)>) {
    for (entity, sink, menusound) in &qs {
        let vol = sink.volume();
        let v = if menusound.despawn {
            vol / 1.02
        } else {
            const DESIRED_VOLUME: f32 = 0.2;
            const STEPS: f32 = 120.0;
            if vol < DESIRED_VOLUME / 2.0 {
                vol * 1.02
            } else {
                (vol * STEPS + DESIRED_VOLUME) / (STEPS + 1.0)
            }
        };
        sink.set_volume(v);
        if v < 0.001 {
            commands.entity(entity).despawn_recursive();
            dbg!("Song despawned");
        }
    }
}

pub fn manage_title_song(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut q_sound: Query<&mut MenuSound>,
    app_state: Res<State<root::State>>,
) {
    // Determine the desired song state directly from the current state
    let should_play_song = !matches!(app_state.get(), root::State::InGame);

    // Check if a MenuSound entity already exists
    if let Ok(mut menusound) = q_sound.get_single_mut() {
        // If the song should be despawned and it exists, despawn it
        if !should_play_song && !menusound.despawn {
            // Trigger fade-out and despawn
            menusound.despawn = true;
        } else if should_play_song && menusound.despawn {
            // Song should be playing but is marked for despawn, so reset it
            menusound.despawn = false;
        }
    } else if should_play_song {
        // No MenuSound entity exists, spawn a new one
        commands.spawn(MenuSound::default()).insert(AudioBundle {
            source: asset_server.load("music/unhaunter_intro.ogg"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::new(MUSIC_VOLUME),
                speed: 1.0,
                paused: false,
                spatial: false,
                spatial_scale: None,
            },
        });
    }
}

pub fn setup_ui(mut commands: Commands, handles: Res<root::GameAssets>) {
    let main_color = Color::Srgba(Srgba {
        red: 0.2,
        green: 0.2,
        blue: 0.2,
        alpha: 0.05,
    });
    commands.spawn(NodeBundle {
        style: Style {
            // ```
            // align_self: AlignSelf::Center,
            // ```
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            padding: UiRect {
                left: Val::Percent(10.0 * UI_SCALE),
                right: Val::Percent(10.0 * UI_SCALE),
                top: Val::Percent(5.0 * UI_SCALE),
                bottom: Val::Percent(5.0 * UI_SCALE),
            },
            flex_grow: 1.0,
            ..default()
        },
        ..default()
    }).insert(MenuUI).with_children(|parent| {
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(20.0),
                min_width: Val::Px(0.0),
                min_height: Val::Px(64.0 * UI_SCALE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            ..default()
        }).with_children(|parent| {
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
                height: Val::Percent(20.0 * UI_SCALE),
                ..default()
            },
            ..default()
        });
        parent.spawn(NodeBundle {
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
        }).insert(Menu::default()).with_children(|parent| {
            // text
            parent.spawn(TextBundle::from_section("New Game", TextStyle {
                font: handles.fonts.londrina.w300_light.clone(),
                font_size: 38.0 * UI_SCALE,
                color: MENU_ITEM_COLOR_OFF,
            })).insert(MenuItem::new(MenuID::MapHub));
            parent.spawn(TextBundle::from_section("Manual", TextStyle {
                font: handles.fonts.londrina.w300_light.clone(),
                font_size: 38.0 * UI_SCALE,
                color: MENU_ITEM_COLOR_OFF,
            })).insert(MenuItem::new(MenuID::Manual));

            // parent .spawn(TextBundle::from_section( "Options", TextStyle { font:
            // handles.fonts.londrina.w300_light.clone(), font_size: 38.0 * UI_SCALE, color:
            // MENU_ITEM_COLOR_OFF, }, )) .insert(MenuItem::new(MenuID::Options));
            if !IS_WASM {
                parent.spawn(TextBundle::from_section("Quit", TextStyle {
                    font: handles.fonts.londrina.w300_light.clone(),
                    font_size: 38.0 * UI_SCALE,
                    color: MENU_ITEM_COLOR_OFF,
                })).insert(MenuItem::new(MenuID::Quit));
            }
        });
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                min_height: Val::Percent(20.0 * UI_SCALE),
                flex_grow: 1.0,
                ..default()
            },
            ..default()
        });
        parent.spawn(
            TextBundle::from_section(
                format!(
                    "Unhaunter {}    -   [Arrow Up]/[Arrow Down]: Change menu item   -    [Enter]: Select current item   -   [ESC] Go Back   -   Game Controls: [WASD] [TAB] [Q] [E] [R] [T] [F] [G]",
                    platform::VERSION
                ),
                TextStyle {
                    font: handles.fonts.titillium.w300_light.clone(),
                    font_size: 20.0 * UI_SCALE,
                    color: MENU_ITEM_COLOR_OFF,
                },
            ).with_style(Style {
                padding: UiRect::all(Val::Percent(5.0 * UI_SCALE)),
                align_content: AlignContent::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                justify_self: JustifySelf::Center,
                flex_grow: 0.0,
                flex_shrink: 0.0,
                flex_basis: Val::Px(35.0 * UI_SCALE),
                max_height: Val::Px(35.0 * UI_SCALE),
                ..default()
            }),
        );
    });
    info!("Main menu loaded");
}

pub fn item_logic(mut q: Query<(&mut MenuItem, &mut Text)>, qmenu: Query<&Menu>) {
    for (mut mitem, mut text) in q.iter_mut() {
        for menu in qmenu.iter() {
            mitem.highlighted = menu.selected == mitem.identifier;
        }
        for section in text.sections.iter_mut() {
            let new_color = if mitem.highlighted {
                MENU_ITEM_COLOR_ON
            } else {
                MENU_ITEM_COLOR_OFF
            };
            if new_color != section.style.color {
                section.style.color = new_color;
            }
        }
    }
}

pub fn keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut Menu>,
    mut ev_menu: EventWriter<MenuEvent>,
) {
    for mut menu in q.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            menu.previous_item();
        } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            menu.next_item();
        } else if keyboard_input.just_pressed(KeyCode::Enter) {
            ev_menu.send(MenuEvent(menu.selected));
        }
    }
}

pub fn menu_event(
    mut ev_menu: EventReader<MenuEvent>,
    mut exit: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<root::State>>,
) {
    for event in ev_menu.read() {
        warn!("Main Menu Event: {:?}", event.0);
        match event.0 {
            MenuID::MapHub => {
                // Transition to the Map Hub state
                next_state.set(root::State::MapHub);
            }
            MenuID::Manual => {
                // Transition to the Manual state
                next_state.set(root::State::UserManual);
            }
            MenuID::_Options => {}
            MenuID::Quit => {
                exit.send(AppExit::Success);
            }
        }
    }
}
