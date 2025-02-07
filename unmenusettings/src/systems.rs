use crate::components::{
    AudioSettingSelected, AudioSettingsValue, MenuEvBack, MenuEvent, MenuItem,
    MenuSettingClassSelected, SaveAudioSetting, SettingsMenu, SettingsState,
};
use crate::menu_ui::setup_ui_main_cat;
use crate::menus::{AudioSettingsMenu, MenuSettingsLevel1};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::colors::{MENU_ITEM_COLOR_OFF, MENU_ITEM_COLOR_ON};
use uncore::states::AppState;
use uncore::types::root::game_assets::GameAssets;
use unsettings::audio::AudioSettings;

pub fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut SettingsMenu>,
    menu_items: Query<&MenuItem>,
    mut ev_menu: EventWriter<MenuEvent>,
) {
    let mut menu = q.single_mut();
    let max_menu_idx = menu_items.iter().count();
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        if menu.selected_item_idx == 0 {
            menu.selected_item_idx = max_menu_idx - 1;
        } else {
            menu.selected_item_idx -= 1;
        }
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        menu.selected_item_idx = (menu.selected_item_idx + 1) % max_menu_idx;
    } else if keyboard_input.just_pressed(KeyCode::Enter) {
        if let Some(menu_item) = menu_items
            .iter()
            .find(|item| item.idx == menu.selected_item_idx)
        {
            ev_menu.send(menu_item.on_activate);
        }
    } else if keyboard_input.just_pressed(KeyCode::Escape) {
        ev_menu.send(MenuEvent::Back(MenuEvBack));
    }
}

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
        // Note: these are now handled on creation and disabled items no longer have MenuItem component.
        // let alpha = if item.on_activate.is_none() { 0.2 } else { 1.0 };
        // text_color.0 = color.with_alpha(alpha);
        text_color.0 = color;
    }
}

pub fn menu_routing_system(
    mut ev_menu: EventReader<MenuEvent>,
    mut ev_back: EventWriter<MenuEvBack>,
    mut ev_class: EventWriter<MenuSettingClassSelected>,
    mut ev_audio_setting: EventWriter<AudioSettingSelected>,
    mut ev_save_audio_setting: EventWriter<SaveAudioSetting>,
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
) {
    for ev in events.read() {
        warn!("Menu Setting Class Selected: {:?}", ev.menu);

        if matches!(ev.menu, MenuSettingsLevel1::Audio) {
            let menu_items = AudioSettingsMenu::iter_events(&audio_settings);
            setup_ui_main_cat(
                &mut commands,
                &handles,
                &qtui,
                "Audio Settings",
                &menu_items,
            );
            next_state.set(SettingsState::Lv2List);
        } else {
            // Just a demo for the ones that don't work yet.
            let menu_items = MenuSettingsLevel1::iter_events();
            setup_ui_main_cat(
                &mut commands,
                &handles,
                &qtui,
                "Broken Settings",
                &menu_items[..2],
            );
        }
    }
}

pub fn menu_audio_setting_selected(
    mut commands: Commands,
    mut events: EventReader<AudioSettingSelected>,
    mut next_state: ResMut<NextState<SettingsState>>,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
) {
    for ev in events.read() {
        warn!("Audio Setting Selected: {:?}", ev.setting);

        let menu_items = ev.setting.iter_events_item();
        setup_ui_main_cat(
            &mut commands,
            &handles,
            &qtui,
            ev.setting.to_string(),
            &menu_items,
        );
        next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Audio));
    }
}

pub fn menu_save_audio_setting(
    mut events: EventReader<SaveAudioSetting>,
    mut ev_back: EventWriter<MenuEvBack>,
    mut audio_settings: ResMut<Persistent<AudioSettings>>,
) {
    use AudioSettingsValue as v;

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
