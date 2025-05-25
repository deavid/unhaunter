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
            .add_event::<MenuEvent>()
            .add_event::<MenuEvBack>()
            .add_event::<MenuSettingClassSelected>()
            .add_event::<AudioSettingSelected>()
            .add_event::<SaveAudioSetting>()
            .add_event::<GameplaySettingSelected>()
            .add_event::<SaveGameplaySetting>();

        // Setup UI systems
        menu_ui::app_setup(app);

        // Setup update systems
        systems::app_setup(app);
    }
}
