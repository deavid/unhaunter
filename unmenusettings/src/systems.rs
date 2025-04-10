use crate::components::{
    AudioSettingSelected, GameplaySettingSelected, MenuEvBack, MenuEvent, MenuItem,
    MenuSettingClassSelected, MenuType, SaveAudioSetting, SaveGameplaySetting, SettingsMenu,
    SettingsState, SettingsStateTimer,
};
use crate::menu_ui::setup_ui_main_cat;
use crate::menus::{AudioSettingsMenu, GameplaySettingsMenu, MenuSettingsLevel1};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::colors::{MENU_ITEM_COLOR_OFF, MENU_ITEM_COLOR_ON};
use uncore::states::AppState;
use uncore::types::root::game_assets::GameAssets;
use uncoremenu::components::{MenuItemInteractive, MenuRoot};
use uncoremenu::systems::MenuItemClicked;
use uncoremenu::templates;
use unsettings::audio::AudioSettings;
use unsettings::game::GameplaySettings;

pub fn item_highlight_system(
    menu: Query<&SettingsMenu>,
    mut menu_items: Query<(&MenuItem, &mut TextColor)>,
) {
    let menu = menu.single(); // Assuming you have only one Menu component
    for (item, mut text_color) in &mut menu_items {
        let is_selected = item.idx == menu.selected_item_idx;
        let color = if is_selected {
            MENU_ITEM_COLOR_ON
        } else {
            MENU_ITEM_COLOR_OFF
        };
        text_color.0 = color;
    }
}

pub fn menu_routing_system(
    mut ev_menu: EventReader<MenuEvent>,
    mut ev_back: EventWriter<MenuEvBack>,
    mut ev_class: EventWriter<MenuSettingClassSelected>,
    mut ev_audio_setting: EventWriter<AudioSettingSelected>,
    mut ev_save_audio_setting: EventWriter<SaveAudioSetting>,
    mut ev_game_setting: EventWriter<GameplaySettingSelected>,
    mut ev_save_game_setting: EventWriter<SaveGameplaySetting>,
) {
    for ev in ev_menu.read() {
        match ev {
            MenuEvent::Back(menu_back) => {
                ev_back.send(menu_back.to_owned());
            }
            MenuEvent::None => {}
            MenuEvent::SettingClassSelected(menu_settings_level1) => {
                ev_class.send(MenuSettingClassSelected {
                    menu: menu_settings_level1.to_owned(),
                });
            }
            MenuEvent::EditAudioSetting(audio_settings_menu) => {
                ev_audio_setting.send(AudioSettingSelected {
                    setting: *audio_settings_menu,
                });
            }
            MenuEvent::SaveAudioSetting(setting_value) => {
                ev_save_audio_setting.send(SaveAudioSetting {
                    value: *setting_value,
                });
            }
            MenuEvent::EditGameplaySetting(gameplay_settings_menu) => {
                ev_game_setting.send(GameplaySettingSelected {
                    setting: *gameplay_settings_menu,
                });
            }
            MenuEvent::SaveGameplaySetting(setting_value) => {
                ev_save_game_setting.send(SaveGameplaySetting {
                    value: *setting_value,
                });
            }
        }
    }
}

pub fn menu_back_event(
    mut events: EventReader<MenuEvBack>,
    mut next_state: ResMut<NextState<SettingsState>>,
    mut app_next_state: ResMut<NextState<AppState>>,
    settings_state: Res<State<SettingsState>>,
    mut ev_menu: EventWriter<MenuSettingClassSelected>,
    mut commands: Commands,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
) {
    for _ev in events.read() {
        match settings_state.get() {
            SettingsState::Lv1ClassSelection => {
                app_next_state.set(AppState::MainMenu);
                next_state.set(SettingsState::default());
            }
            SettingsState::Lv2List => {
                next_state.set(SettingsState::Lv1ClassSelection);
                // Redraw Main Menu:
                let menu_items = MenuSettingsLevel1::iter_events();
                setup_ui_main_cat(&mut commands, &handles, &qtui, "Settings", &menu_items);
            }
            SettingsState::Lv3ValueEdit(menu) => {
                ev_menu.send(MenuSettingClassSelected { menu: *menu });
            }
        }
    }
}

pub fn menu_settings_class_selected(
    mut commands: Commands,
    mut events: EventReader<MenuSettingClassSelected>,
    mut next_state: ResMut<NextState<SettingsState>>,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
    audio_settings: Res<Persistent<AudioSettings>>,
    game_settings: Res<Persistent<GameplaySettings>>,
) {
    for ev in events.read() {
        warn!("Menu Setting Class Selected: {:?}", ev.menu);
        match ev.menu {
            MenuSettingsLevel1::Audio => {
                let menu_items = AudioSettingsMenu::iter_events(&audio_settings);
                setup_ui_main_cat(
                    &mut commands,
                    &handles,
                    &qtui,
                    "Audio Settings",
                    &menu_items,
                );
                next_state.set(SettingsState::Lv2List);
            }
            MenuSettingsLevel1::Gameplay => {
                let menu_items = GameplaySettingsMenu::iter_events(&game_settings);
                setup_ui_main_cat(
                    &mut commands,
                    &handles,
                    &qtui,
                    "Gameplay Settings",
                    &menu_items,
                );
                next_state.set(SettingsState::Lv2List);
            }
            MenuSettingsLevel1::Video => todo!(),
            MenuSettingsLevel1::Profile => todo!(),
        }
    }
}

pub fn menu_audio_setting_selected(
    mut commands: Commands,
    mut events: EventReader<AudioSettingSelected>,
    mut next_state: ResMut<NextState<SettingsState>>,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
    audio_settings: Res<Persistent<AudioSettings>>,
) {
    for ev in events.read() {
        warn!("Audio Setting Selected: {:?}", ev.setting);

        let menu_items = ev.setting.iter_events_item(&audio_settings);

        // Clean up old UI
        for e in qtui.iter() {
            commands.entity(e).despawn_recursive();
        }

        // Create new UI with uncoremenu templates
        commands
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            })
            .insert(SettingsMenu {
                menu_type: MenuType::SettingEdit,
                selected_item_idx: 0,
            })
            .with_children(|parent| {
                // Background
                templates::create_background(parent, &handles);

                // Logo
                templates::create_logo(parent, &handles);

                // Create breadcrumb navigation with title - show the full path
                templates::create_breadcrumb_navigation(
                    parent,
                    &handles,
                    "Audio Settings",
                    ev.setting.to_string()
                );

                // Create content area for settings items
                let mut content_area = templates::create_selectable_content_area(
                    parent,
                    &handles,
                    0 // Initial selection
                );
                content_area.insert(MenuRoot {
                    selected_item: 0,
                });

                // Add a column container inside the content area for vertical layout
                content_area.with_children(|content| {
                    content
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            justify_content: JustifyContent::FlexStart,
                            overflow: Overflow::scroll_y(),
                            ..default()
                        })
                        .with_children(|menu_list| {
                            let mut idx = 0;

                            // Add each menu item
                            for (item_text, event) in menu_items.iter() {
                                if !event.is_none() {
                                    templates::create_content_item(
                                        menu_list,
                                        item_text,
                                        idx,
                                        idx == 0, // First item selected by default
                                        &handles
                                    )
                                    .insert(MenuItem::new(idx, *event));
                                    idx += 1;
                                }
                            }

                            // Add "Go Back" option
                            templates::create_content_item(
                                menu_list,
                                "Go Back",
                                idx,
                                false,
                                &handles
                            )
                            .insert(MenuItem::new(idx, MenuEvent::Back(MenuEvBack)));
                        });
                });

                // Help text
                templates::create_help_text(
                    parent,
                    &handles,
                    Some("[Up]/[Down] arrows to navigate. Press [Enter] to select or [Escape] to go back".to_string())
                );
            });

        next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Audio));
    }
}

pub fn menu_save_audio_setting(
    mut events: EventReader<SaveAudioSetting>,
    mut ev_back: EventWriter<MenuEvBack>,
    mut audio_settings: ResMut<Persistent<AudioSettings>>,
) {
    use unsettings::audio::AudioSettingsValue as v;

    for ev in events.read() {
        warn!("Save Audio Setting: {:?}", ev.value);
        match ev.value {
            v::volume_master(audio_level) => {
                audio_settings.volume_master = audio_level;
            }
            v::volume_music(audio_level) => {
                audio_settings.volume_music = audio_level;
            }
            v::volume_effects(audio_level) => {
                audio_settings.volume_effects = audio_level;
            }
            v::volume_ambient(audio_level) => {
                audio_settings.volume_ambient = audio_level;
            }
            v::volume_voice_chat(audio_level) => {
                audio_settings.volume_voice_chat = audio_level;
            }
            v::sound_output(sound_output) => {
                audio_settings.sound_output = sound_output;
            }
            v::audio_positioning(audio_positioning) => {
                audio_settings.audio_positioning = audio_positioning;
            }
            v::feedback_delay(feedback_delay) => {
                audio_settings.feedback_delay = feedback_delay;
            }
            v::feedback_eq(feedback_eq) => {
                audio_settings.feedback_eq = feedback_eq;
            }
        }
        if let Err(e) = audio_settings.persist() {
            error!("Error persisting Audio Settings: {e:?}");
        }
        ev_back.send(MenuEvBack);
    }
}

pub fn menu_gameplay_setting_selected(
    mut commands: Commands,
    mut events: EventReader<GameplaySettingSelected>,
    mut next_state: ResMut<NextState<SettingsState>>,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
    game_settings: Res<Persistent<GameplaySettings>>,
) {
    for ev in events.read() {
        warn!("Gameplay Setting Selected: {:?}", ev.setting);

        let menu_items = ev.setting.iter_events_item(&game_settings);

        // Clean up old UI
        for e in qtui.iter() {
            commands.entity(e).despawn_recursive();
        }

        // Create new UI with uncoremenu templates
        commands
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            })
            .insert(SettingsMenu {
                menu_type: MenuType::SettingEdit,
                selected_item_idx: 0,
            })
            .with_children(|parent| {
                // Background
                templates::create_background(parent, &handles);

                // Logo

                templates::create_logo(parent, &handles);

                // Create breadcrumb navigation with title - show the full path
                templates::create_breadcrumb_navigation(
                    parent,
                    &handles,
                    "Gameplay Settings",
                    ev.setting.to_string(),
                );

                // Create content area for settings items
                let mut content_area = templates::create_selectable_content_area(
                    parent,
                    &handles,
                    0 // Initial selection
                );
                content_area.insert(MenuRoot {
                    selected_item: 0,
                });

                // Add a column container inside the content area for vertical layout
                content_area.with_children(|content| {
                    content
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            justify_content: JustifyContent::FlexStart,
                            overflow: Overflow::scroll_y(),
                            ..default()
                        })
                        .with_children(|menu_list| {
                            let mut idx = 0;

                            // Add each menu item
                            for (item_text, event) in menu_items.iter() {
                                if !event.is_none() {
                                    templates::create_content_item(
                                        menu_list,
                                        item_text,
                                        idx,
                                        idx == 0, // First item selected by default
                                        &handles
                                    )
                                    .insert(MenuItem::new(idx, *event));
                                    idx += 1;
                                }
                            }

                            // Add "Go Back" option
                            templates::create_content_item(
                                menu_list,
                                "Go Back",
                                idx,
                                false,
                                &handles
                            )
                            .insert(MenuItem::new(idx, MenuEvent::Back(MenuEvBack)));
                        });
                });

                // Help text
                templates::create_help_text(
                    parent,
                    &handles,
                    Some("[Up]/[Down] arrows to navigate. Press [Enter] to select or [Escape] to go back".to_string())
                );
            });

        next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Gameplay));
    }
}

pub fn menu_save_gameplay_setting(
    mut events: EventReader<SaveGameplaySetting>,
    mut ev_back: EventWriter<MenuEvBack>,
    mut gameplay_settings: ResMut<Persistent<GameplaySettings>>,
) {
    use unsettings::game::GameplaySettingsValue as v;

    for ev in events.read() {
        warn!("Save Gameplay Setting: {:?}", ev.value);
        match ev.value {
            v::movement_style(movement_style) => {
                gameplay_settings.movement_style = movement_style;
            }
            v::camera_controls(camera_controls) => {
                gameplay_settings.camera_controls = camera_controls;
            }
            v::character_controls(character_controls) => {
                gameplay_settings.character_controls = character_controls;
            }
        }
        if let Err(e) = gameplay_settings.persist() {
            error!("Error persisting Gameplay Settings: {e:?}");
        }
        ev_back.send(MenuEvBack);
    }
}

pub fn menu_integration_system(
    mut menu_clicks: EventReader<MenuItemClicked>,
    mut menu_events: EventWriter<MenuEvent>,
    menu_items: Query<(&MenuItem, &MenuItemInteractive)>,
    state_timer: Query<&SettingsStateTimer>,
) {
    // Define a small grace period to ignore events from previous state
    const GRACE_PERIOD_SECS: f32 = 0.1;

    // Get time since state entered
    if let Ok(timer) = state_timer.get_single() {
        let time_in_state = timer.state_entered_at.elapsed().as_secs_f32();

        // Ignore events that happened too soon after state transition
        if time_in_state < GRACE_PERIOD_SECS {
            menu_clicks.clear();
            return;
        }

        for click_event in menu_clicks.read() {
            warn!("Settings menu received click event: {:?}", click_event);
            let clicked_idx = click_event.0;

            // Find the menu item with this index
            if let Some((menu_item, _)) = menu_items
                .iter()
                .find(|(_, interactive)| interactive.identifier == clicked_idx)
            {
                // Send the corresponding menu event
                menu_events.send(menu_item.on_activate);
                warn!("Activating menu item: {:?}", menu_item.on_activate);
            } else {
                warn!("No menu item found with index {}", clicked_idx);
            }
        }
        menu_clicks.clear();
    }
}

/// Handles the ESC key events from the core menu system
pub fn handle_escape(
    mut escape_events: EventReader<uncoremenu::systems::MenuEscapeEvent>,
    mut menu_events: EventWriter<MenuEvent>,
) {
    if !escape_events.is_empty() {
        // If ESC was pressed, send a Back event
        menu_events.send(MenuEvent::Back(MenuEvBack));
        escape_events.clear();
    }
}
