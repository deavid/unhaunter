use crate::components::{
    AudioSettingSelected, GameplaySettingSelected, MenuEvBack, MenuEvent, MenuSettingClassSelected,
    SaveAudioSetting, SaveGameplaySetting, SettingsState,
};
use crate::{menu_ui, systems};
use bevy::prelude::*;

pub struct UnhaunterMenuSettingsPlugin;

impl Plugin for UnhaunterMenuSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SettingsState>()
            .add_message::<MenuEvent>()
            .add_message::<MenuEvBack>()
            .add_message::<MenuSettingClassSelected>()
            .add_message::<AudioSettingSelected>()
            .add_message::<SaveAudioSetting>()
            .add_message::<GameplaySettingSelected>()
            .add_message::<SaveGameplaySetting>();

        // Setup UI systems
        menu_ui::app_setup(app);

        // Setup update systems
        systems::app_setup(app);
    }
}
