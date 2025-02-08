use crate::components::{
    AudioSettingSelected, GameplaySettingSelected, MenuEvBack, MenuEvent, MenuSettingClassSelected,
    SaveAudioSetting, SaveGameplaySetting, SettingsState,
};
use crate::{menu_ui, systems};
use bevy::prelude::*;
use uncore::states::AppState;

pub struct UnhaunterMenuSettingsPlugin;

impl Plugin for UnhaunterMenuSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SettingsState>()
            .add_systems(
                OnEnter(AppState::SettingsMenu),
                (menu_ui::setup_ui_cam, menu_ui::setup_ui_main_cat_system).chain(),
            )
            .add_systems(OnExit(AppState::SettingsMenu), menu_ui::cleanup)
            .add_systems(
                Update,
                (
                    systems::handle_input,
                    systems::item_highlight_system,
                    systems::menu_routing_system,
                    systems::menu_back_event,
                    systems::menu_settings_class_selected,
                    systems::menu_audio_setting_selected,
                    systems::menu_save_audio_setting,
                    systems::menu_gameplay_setting_selected,
                    systems::menu_save_gameplay_setting,
                )
                    .run_if(in_state(AppState::SettingsMenu)),
            )
            .add_event::<MenuEvent>()
            .add_event::<MenuEvBack>()
            .add_event::<MenuSettingClassSelected>()
            .add_event::<AudioSettingSelected>()
            .add_event::<SaveAudioSetting>()
            .add_event::<GameplaySettingSelected>()
            .add_event::<SaveGameplaySetting>();
    }
}
