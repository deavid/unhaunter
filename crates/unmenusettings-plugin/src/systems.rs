use crate::components::{
    AudioSettingSelected, GameplaySettingSelected, MenuEvBack, MenuEvent, MenuItem,
    MenuSettingClassSelected, SaveAudioSetting, SaveGameplaySetting, SaveVideoSetting,
    SettingsMenu, SettingsState, SettingsStateTimer, VideoSettingSelected,
};
use crate::menu_ui::setup_ui_main_cat;
use crate::menus::{
    AudioSettingsMenu, GameplaySettingsMenu, MenuSettingsLevel1, VideoSettingsMenu,
};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use unfoundation_core::colors::{MENU_ITEM_COLOR_OFF, MENU_ITEM_COLOR_ON};
use unmenu_core::components::{MenuItemInteractive, MenuMouseTracker, MenuRoot};
use unmenu_core::events::MenuItemClicked;
use unmenu_core::templates;
use unsettings_core::audio::AudioSettings;
use unsettings_core::game::GameplaySettings;
use unsettings_core::video::VideoSettings;
use untypes_core::states::AppState;
use unui_core::assets::UiAssets;

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            item_highlight_system,
            menu_routing_system,
            menu_back_event,
            menu_settings_class_selected,
            menu_audio_setting_selected,
            menu_save_audio_setting,
            menu_gameplay_setting_selected,
            menu_save_gameplay_setting,
            menu_video_setting_selected,
            menu_save_video_setting,
            menu_integration_system,
            handle_escape,
        )
            .run_if(in_state(AppState::SettingsMenu)),
    )
    .add_message::<MenuEvent>()
    .add_message::<MenuEvBack>()
    .add_message::<MenuSettingClassSelected>()
    .add_message::<AudioSettingSelected>()
    .add_message::<SaveAudioSetting>()
    .add_message::<GameplaySettingSelected>()
    .add_message::<SaveGameplaySetting>()
    .add_message::<VideoSettingSelected>()
    .add_message::<SaveVideoSetting>();
}

fn item_highlight_system(
    menu: Query<&SettingsMenu>,
    mut menu_items: Query<(&MenuItem, &mut TextColor)>,
) {
    let Ok(menu) = menu.single() else {
        return;
    }; // Assuming you have only one Menu component
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

fn menu_routing_system(
    mut ev_menu: MessageReader<MenuEvent>,
    mut ev_back: MessageWriter<MenuEvBack>,
    mut ev_class: MessageWriter<MenuSettingClassSelected>,
    mut ev_audio_setting: MessageWriter<AudioSettingSelected>,
    mut ev_save_audio_setting: MessageWriter<SaveAudioSetting>,
    mut ev_video_setting: MessageWriter<VideoSettingSelected>,
    mut ev_save_video_setting: MessageWriter<SaveVideoSetting>,
    mut ev_game_setting: MessageWriter<GameplaySettingSelected>,
    mut ev_save_game_setting: MessageWriter<SaveGameplaySetting>,
) {
    for ev in ev_menu.read() {
        match ev {
            MenuEvent::Back(menu_back) => {
                ev_back.write(menu_back.to_owned());
            }
            MenuEvent::None => {}
            MenuEvent::SettingClassSelected(menu_settings_level1) => {
                ev_class.write(MenuSettingClassSelected {
                    menu: menu_settings_level1.to_owned(),
                });
            }
            MenuEvent::EditAudioSetting(audio_settings_menu) => {
                ev_audio_setting.write(AudioSettingSelected {
                    setting: *audio_settings_menu,
                });
            }
            MenuEvent::SaveAudioSetting(setting_value) => {
                ev_save_audio_setting.write(SaveAudioSetting {
                    value: *setting_value,
                });
            }
            MenuEvent::EditVideoSetting(video_settings_menu) => {
                ev_video_setting.write(VideoSettingSelected {
                    setting: *video_settings_menu,
                });
            }
            MenuEvent::SaveVideoSetting(setting_value) => {
                ev_save_video_setting.write(SaveVideoSetting {
                    value: *setting_value,
                });
            }
            MenuEvent::EditGameplaySetting(gameplay_settings_menu) => {
                ev_game_setting.write(GameplaySettingSelected {
                    setting: *gameplay_settings_menu,
                });
            }
            MenuEvent::SaveGameplaySetting(setting_value) => {
                ev_save_game_setting.write(SaveGameplaySetting {
                    value: *setting_value,
                });
            }
        }
    }
}

fn menu_back_event(
    mut events: MessageReader<MenuEvBack>,
    mut next_state: ResMut<NextState<SettingsState>>,
    mut app_next_state: ResMut<NextState<AppState>>,
    settings_state: Res<State<SettingsState>>,
    mut ev_menu: MessageWriter<MenuSettingClassSelected>,
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
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
                setup_ui_main_cat(&mut commands, &ui_assets, &qtui, "Settings", &menu_items);
            }
            SettingsState::Lv3ValueEdit(menu) => {
                ev_menu.write(MenuSettingClassSelected { menu: *menu });
            }
        }
    }
}

fn menu_settings_class_selected(
    mut commands: Commands,
    mut events: MessageReader<MenuSettingClassSelected>,
    mut next_state: ResMut<NextState<SettingsState>>,
    ui_assets: Res<UiAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
    audio_settings: Res<Persistent<AudioSettings>>,
    game_settings: Res<Persistent<GameplaySettings>>,
    video_settings: Res<Persistent<VideoSettings>>,
) {
    for ev in events.read() {
        debug!("Menu Setting Class Selected: {:?}", ev.menu);
        match ev.menu {
            MenuSettingsLevel1::Audio => {
                let menu_items = AudioSettingsMenu::iter_events(&audio_settings);
                setup_ui_main_cat(
                    &mut commands,
                    &ui_assets,
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
                    &ui_assets,
                    &qtui,
                    "Gameplay Settings",
                    &menu_items,
                );
                next_state.set(SettingsState::Lv2List);
            }
            MenuSettingsLevel1::Video => {
                let menu_items = VideoSettingsMenu::iter_events(&video_settings);
                setup_ui_main_cat(
                    &mut commands,
                    &ui_assets,
                    &qtui,
                    "Video Settings",
                    &menu_items,
                );
                next_state.set(SettingsState::Lv2List);
            }
            MenuSettingsLevel1::Profile => todo!(),
        }
    }
}

fn menu_video_setting_selected(
    mut events: MessageReader<VideoSettingSelected>,
    mut next_state: ResMut<NextState<SettingsState>>,
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
    video_settings: Res<Persistent<VideoSettings>>,
) {
    for ev in events.read() {
        let menu_items = ev.setting.iter_events_item(&video_settings);

        for e in qtui.iter() {
            commands.entity(e).despawn();
        }

        commands
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            })
            .insert(SettingsMenu {
                selected_item_idx: 0,
            })
            .with_children(|parent| {
                templates::create_background(parent, &ui_assets);
                templates::create_logo(parent, &ui_assets);
                templates::create_breadcrumb_navigation(parent, &ui_assets, "Video Settings", ev.setting.to_string());

                let mut content_area = templates::create_selectable_content_area(parent, &ui_assets, 0);

                content_area.insert(MenuMouseTracker::default());
                content_area.insert(MenuRoot { selected_item: 0 });

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
                            for (item_text, event) in menu_items.iter() {
                                if !event.is_none() {
                                    templates::create_content_item(
                                        menu_list,
                                        item_text,
                                        idx,
                                        idx == 0,
                                        &ui_assets,
                                    )
                                    .insert(MenuItem::new(idx, *event));
                                    idx += 1;
                                }
                            }
                            templates::create_content_item(
                                menu_list,
                                "Go Back",
                                idx,
                                false,
                                &ui_assets,
                            )
                            .insert(MenuItem::new(idx, MenuEvent::Back(MenuEvBack)));
                        });
                });

                templates::create_help_text(
                    parent,
                    &ui_assets,
                    Some("[Up]/[Down] arrows to navigate. Press [Enter] to select or [Escape] to go back".to_string())
                );
            });

        next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Video));
    }
}

fn menu_save_video_setting(
    mut events: MessageReader<SaveVideoSetting>,
    mut ev_back: MessageWriter<MenuEvBack>,
    mut video_settings: ResMut<Persistent<VideoSettings>>,
) {
    use unsettings_core::video::VideoSettingsValue as v;

    for ev in events.read() {
        debug!("Save Video Setting: {:?}", ev.value);
        match ev.value {
            v::window_size(s) => video_settings.window_size = s,
            v::aspect_ratio(s) => video_settings.aspect_ratio = s,
            v::ui_scale(s) => video_settings.ui_scale = s,
            v::font_scale(s) => video_settings.font_scale = s,
            v::max_upscale_factor(s) => video_settings.max_upscale_factor = s,
        }
        if let Err(e) = video_settings.persist() {
            error!("Error persisting Video Settings: {e:?}");
        }
        ev_back.write(MenuEvBack);
    }
}

fn menu_audio_setting_selected(
    mut commands: Commands,
    mut events: MessageReader<AudioSettingSelected>,
    mut next_state: ResMut<NextState<SettingsState>>,
    ui_assets: Res<UiAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
    audio_settings: Res<Persistent<AudioSettings>>,
) {
    for ev in events.read() {
        debug!("Audio Setting Selected: {:?}", ev.setting);

        let menu_items = ev.setting.iter_events_item(&audio_settings);

        // Clean up old UI
        for e in qtui.iter() {
            commands.entity(e).despawn();
        }

        // Create new UI with unmenu_core templates
        commands
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            })
            .insert(SettingsMenu {
                selected_item_idx: 0,
            })
            .with_children(|parent| {
                // Background
                templates::create_background(parent, &ui_assets);

                // Logo
                templates::create_logo(parent, &ui_assets);

                // Create breadcrumb navigation with title - show the full path
                templates::create_breadcrumb_navigation(
                    parent,
                    &ui_assets,
                    "Audio Settings",
                    ev.setting.to_string()
                );

                // Create content area for settings items
                let mut content_area = templates::create_selectable_content_area(
                    parent,
                    &ui_assets,
                    0 // Initial selection
                );

                // Add mouse tracker to prevent unwanted initial hover selection
                content_area.insert(MenuMouseTracker::default());

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
                                        &ui_assets
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
                                &ui_assets
                            )
                            .insert(MenuItem::new(idx, MenuEvent::Back(MenuEvBack)));
                        });
                });

                // Help text
                templates::create_help_text(
                    parent,
                    &ui_assets,
                    Some("[Up]/[Down] arrows to navigate. Press [Enter] to select or [Escape] to go back".to_string())
                );
            });

        next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Audio));
    }
}

fn menu_save_audio_setting(
    mut events: MessageReader<SaveAudioSetting>,
    mut ev_back: MessageWriter<MenuEvBack>,
    mut audio_settings: ResMut<Persistent<AudioSettings>>,
) {
    use unsettings_core::audio::AudioSettingsValue as v;

    for ev in events.read() {
        debug!("Save Audio Setting: {:?}", ev.value);
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
        ev_back.write(MenuEvBack);
    }
}

fn menu_gameplay_setting_selected(
    mut commands: Commands,
    mut events: MessageReader<GameplaySettingSelected>,
    mut next_state: ResMut<NextState<SettingsState>>,
    ui_assets: Res<UiAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
    game_settings: Res<Persistent<GameplaySettings>>,
) {
    for ev in events.read() {
        debug!("Gameplay Setting Selected: {:?}", ev.setting);

        let menu_items = ev.setting.iter_events_item(&game_settings);

        // Clean up old UI
        for e in qtui.iter() {
            commands.entity(e).despawn();
        }

        // Create new UI with unmenu_core templates
        commands
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            })
            .insert(SettingsMenu {
                selected_item_idx: 0,
            })
            .with_children(|parent| {
                // Background
                templates::create_background(parent, &ui_assets);

                // Logo

                templates::create_logo(parent, &ui_assets);

                // Create breadcrumb navigation with title - show the full path
                templates::create_breadcrumb_navigation(
                    parent,
                    &ui_assets,
                    "Gameplay Settings",
                    ev.setting.to_string(),
                );

                // Create content area for settings items
                let mut content_area = templates::create_selectable_content_area(
                    parent,
                    &ui_assets,
                    0 // Initial selection
                );

                // Add mouse tracker to prevent unwanted initial hover selection
                content_area.insert(MenuMouseTracker::default());

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
                                        &ui_assets
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
                                &ui_assets
                            )
                            .insert(MenuItem::new(idx, MenuEvent::Back(MenuEvBack)));
                        });
                });

                // Help text
                templates::create_help_text(
                    parent,
                    &ui_assets,
                    Some("[Up]/[Down] arrows to navigate. Press [Enter] to select or [Escape] to go back".to_string())
                );
            });

        next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Gameplay));
    }
}

fn menu_save_gameplay_setting(
    mut events: MessageReader<SaveGameplaySetting>,
    mut ev_back: MessageWriter<MenuEvBack>,
    mut gameplay_settings: ResMut<Persistent<GameplaySettings>>,
) {
    use unsettings_core::game::GameplaySettingsValue as v;

    for ev in events.read() {
        debug!("Save Gameplay Setting: {:?}", ev.value);
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
        ev_back.write(MenuEvBack);
    }
}

fn menu_integration_system(
    mut menu_clicks: MessageReader<MenuItemClicked>,
    mut menu_events: MessageWriter<MenuEvent>,
    menu_items: Query<(&MenuItem, &MenuItemInteractive)>,
    state_timer: Query<&SettingsStateTimer>,
) {
    // Define a small grace period to ignore events from previous state
    const GRACE_PERIOD_SECS: f32 = 0.1;

    // Get time since state entered
    if let Ok(timer) = state_timer.single() {
        let time_in_state = timer.state_entered_at.elapsed().as_secs_f32();

        // Ignore events that happened too soon after state transition
        if time_in_state < GRACE_PERIOD_SECS {
            menu_clicks.clear();
            return;
        }

        for click_event in menu_clicks.read() {
            if click_event.state != AppState::SettingsMenu {
                warn!(
                    "MenuItemClicked event received in state: {:?}",
                    click_event.state
                );
                continue;
            }
            trace!("Settings menu received click event: {:?}", click_event);
            let clicked_idx = click_event.pos;

            // Find the menu item with this index
            if let Some((menu_item, _)) = menu_items
                .iter()
                .find(|(_, interactive)| interactive.identifier == clicked_idx)
            {
                // Send the corresponding menu event
                menu_events.write(menu_item.on_activate);
                debug!("Activating menu item: {:?}", menu_item.on_activate);
            } else {
                warn!("No menu item found with index {}", clicked_idx);
            }
        }
        menu_clicks.clear();
    }
}

/// Handles the ESC key events from the core menu system
fn handle_escape(
    mut escape_events: MessageReader<unmenu_core::events::MenuEscapeEvent>,
    mut menu_events: MessageWriter<MenuEvent>,
) {
    if !escape_events.is_empty() {
        // If ESC was pressed, send a Back event
        menu_events.write(MenuEvent::Back(MenuEvBack));
        escape_events.clear();
    }
}
