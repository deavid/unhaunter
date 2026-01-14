use bevy::prelude::*;
use bevy_platform::time::Instant;
use unsettings_core::{audio::AudioSettingsValue, game::GameplaySettingsValue};

use crate::menus::{AudioSettingsMenu, GameplaySettingsMenu, MenuSettingsLevel1};

// Marker component for the main settings menu UI
#[derive(Component)]
pub(crate) struct SettingsMenu {
    pub selected_item_idx: usize,
}

#[derive(Component)]
pub(crate) struct SCamera;

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, States, Default)]
pub(crate) enum SettingsState {
    /// Selects which Setting file/category to edit in the UI (Audio, Video, etc)
    #[default]
    Lv1ClassSelection,
    /// Lists the settings available in the file for later editing (Volume, Control Type, etc)
    Lv2List,
    /// Allows the user to select a new value for the setting (10% volume, 50% volume, etc)
    Lv3ValueEdit(MenuSettingsLevel1),
}

#[derive(Component)]
pub(crate) struct SettingsStateTimer {
    pub state_entered_at: Instant,
}

#[derive(Component)]
pub(crate) struct MenuItem {
    pub idx: usize,
    pub on_activate: MenuEvent,
}

impl MenuItem {
    pub(crate) fn new(idx: usize, on_activate: MenuEvent) -> Self {
        MenuItem { idx, on_activate }
    }
}

#[derive(Message, Debug, Clone, Copy, Default)]
pub(crate) enum MenuEvent {
    SaveAudioSetting(AudioSettingsValue),
    EditAudioSetting(AudioSettingsMenu),
    SaveGameplaySetting(GameplaySettingsValue),
    EditGameplaySetting(GameplaySettingsMenu),
    SettingClassSelected(MenuSettingsLevel1),
    Back(MenuEvBack),
    #[default]
    None,
}

impl MenuEvent {
    pub(crate) fn is_none(&self) -> bool {
        matches!(self, MenuEvent::None)
    }
}

#[derive(Message, Debug, Clone, Copy)]
pub(crate) struct MenuEvBack;

#[derive(Message, Debug, Clone, Copy)]
pub(crate) struct MenuSettingClassSelected {
    pub menu: MenuSettingsLevel1,
}

#[derive(Message, Debug, Clone, Copy)]
pub(crate) struct AudioSettingSelected {
    pub setting: AudioSettingsMenu,
}

#[derive(Message, Debug, Clone, Copy)]
pub(crate) struct SaveAudioSetting {
    pub value: AudioSettingsValue,
}

#[derive(Message, Debug, Clone, Copy)]
pub(crate) struct GameplaySettingSelected {
    pub setting: GameplaySettingsMenu,
}

#[derive(Message, Debug, Clone, Copy)]
pub(crate) struct SaveGameplaySetting {
    pub value: GameplaySettingsValue,
}
