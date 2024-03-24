use bevy::app::AppExit;
use bevy::prelude::*;

use crate::game::level::LoadLevelEvent;
use crate::root;

use crate::platform::plt::IS_WASM;
use crate::platform::plt::UI_SCALE;

const MENU_ITEM_COLOR_OFF: Color = Color::GRAY;
const MENU_ITEM_COLOR_ON: Color = Color::ORANGE_RED;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuID {
    NewGame,
    Map,
    Options,
    Quit,
}

#[derive(Debug, Copy, Clone, Event)]
pub struct MenuEvent(MenuID);

#[derive(Component)]
pub struct Menu {
    pub selected: MenuID,
    pub map_idx: usize,
    pub map_len: usize,
}

impl Menu {
    pub fn items() -> &'static [MenuID] {
        if IS_WASM {
            &[MenuID::NewGame, MenuID::Map, MenuID::Options]
        } else {
            &[MenuID::NewGame, MenuID::Map, MenuID::Options, MenuID::Quit]
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
    pub fn next_map(&mut self) {
        self.map_idx = (self.map_idx + 1) % self.map_len;
    }
    pub fn previous_map(&mut self) {
        self.map_idx = if self.map_idx < 1 {
            self.map_len - 1
        } else {
            self.map_idx - 1
        };
    }
    pub fn with_len(map_len: usize) -> Self {
        let map_len = map_len.max(1); // Ensure that it is at least 1.
        Self {
            map_len,
            ..default()
        }
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            selected: MenuID::NewGame,
            map_idx: 0,
            map_len: 1,
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
        .add_systems(OnExit(root::State::MainMenu), cleanup);
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ui camera
    let cam = Camera2dBundle::default();
    commands.spawn(cam).insert(MCamera);
    info!("Main menu camera setup");
    commands
        .spawn(AudioBundle {
            source: asset_server.load("music/unhaunter_intro.ogg"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::new(0.1),
                speed: 1.0,
                paused: false,
                spatial: false,
                spatial_scale: None,
            },
        })
        .insert(MenuSound::default());
}

pub fn cleanup(
    mut commands: Commands,
    qc: Query<Entity, With<MCamera>>,
    qm: Query<Entity, With<MenuUI>>,
    mut qs: Query<&mut MenuSound>,
) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }
    // Despawn menu UI if not used
    for ui_entity in qm.iter() {
        commands.entity(ui_entity).despawn_recursive();
    }
    // Despawn Sound
    for mut sound in qs.iter_mut() {
        sound.despawn = true;
    }
}

pub fn despawn_sound(mut commands: Commands, qs: Query<(Entity, &AudioSink, &MenuSound)>) {
    for (entity, sink, menusound) in &qs {
        if !menusound.despawn {
            continue;
        }
        let v = sink.volume() / 1.02;
        sink.set_volume(v);

        if v < 0.001 {
            commands.entity(entity).despawn_recursive();
            dbg!("Song despawned");
        }
    }
}

pub fn setup_ui(mut commands: Commands, handles: Res<root::GameAssets>, maps: Res<root::Maps>) {
    let main_color = Color::Rgba {
        red: 0.2,
        green: 0.2,
        blue: 0.2,
        alpha: 0.05,
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                //    align_self: AlignSelf::Center,
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

                ..default()
            },

            ..default()
        })
        .insert(MenuUI)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
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
                })
                .with_children(|parent| {
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

            parent
                .spawn(NodeBundle {
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
                })
                .insert(Menu::with_len(maps.maps.len()))
                .with_children(|parent| {
                    // text
                    parent
                        .spawn(TextBundle::from_section(
                            "New Game",
                            TextStyle {
                                font: handles.fonts.londrina.w300_light.clone(),
                                font_size: 38.0 * UI_SCALE,
                                color: MENU_ITEM_COLOR_OFF,
                            },
                        ))
                        .insert(MenuItem::new(MenuID::NewGame));
                    parent
                        .spawn(TextBundle::from_section(
                            "Map: ?",
                            TextStyle {
                                font: handles.fonts.londrina.w300_light.clone(),
                                font_size: 38.0 * UI_SCALE,
                                color: MENU_ITEM_COLOR_OFF,
                            },
                        ))
                        .insert(MenuItem::new(MenuID::Map));
                    parent
                        .spawn(TextBundle::from_section(
                            "Options",
                            TextStyle {
                                font: handles.fonts.londrina.w300_light.clone(),
                                font_size: 38.0 * UI_SCALE,
                                color: MENU_ITEM_COLOR_OFF,
                            },
                        ))
                        .insert(MenuItem::new(MenuID::Options));
                    if !IS_WASM {
                        parent
                            .spawn(TextBundle::from_section(
                                "Quit",
                                TextStyle {
                                    font: handles.fonts.londrina.w300_light.clone(),
                                    font_size: 38.0 * UI_SCALE,
                                    color: MENU_ITEM_COLOR_OFF,
                                },
                            ))
                            .insert(MenuItem::new(MenuID::Quit));
                    }
                });
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(20.0 * UI_SCALE),
                    ..default()
                },

                ..default()
            });
        });
    info!("Main menu loaded");
}

pub fn item_logic(
    mut q: Query<(&mut MenuItem, &mut Text)>,
    qmenu: Query<&Menu>,
    maps: Res<root::Maps>,
) {
    for (mut mitem, mut text) in q.iter_mut() {
        let mut map_idx = 0;
        for menu in qmenu.iter() {
            mitem.highlighted = menu.selected == mitem.identifier;
            map_idx = menu.map_idx;
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
            if mitem.identifier == MenuID::Map {
                let map_name = maps
                    .maps
                    .get(map_idx)
                    .map(|x| x.name.clone())
                    .unwrap_or("None".to_string());
                let new_map_name = format!("Map: {}", map_name);
                if section.value != new_map_name {
                    section.value = new_map_name;
                }
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
        if menu.selected == MenuID::Map {
            if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
                menu.previous_map();
            }
            if keyboard_input.just_pressed(KeyCode::ArrowRight) {
                menu.next_map();
            }
        }
    }
}

pub fn menu_event(
    mut ev_menu: EventReader<MenuEvent>,
    mut exit: EventWriter<AppExit>,
    mut ev_load: EventWriter<LoadLevelEvent>,
    q: Query<&Menu>,
    maps: Res<root::Maps>,
) {
    for event in ev_menu.read() {
        warn!("Main Menu Event: {:?}", event.0);
        match event.0 {
            MenuID::NewGame => {
                let map_idx = q.single().map_idx;
                let map_filepath = maps.maps[map_idx].path.clone();
                ev_load.send(LoadLevelEvent { map_filepath });
            }
            MenuID::Map => {}
            MenuID::Options => {}
            MenuID::Quit => {
                exit.send(AppExit);
            }
        }
    }
}
