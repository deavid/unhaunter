use bevy::prelude::*;
use unsettings::audio::{AudioLevel, AudioPositioning, FeedbackDelay, FeedbackEQ, SoundOutput};

use crate::menus::{AudioSettingsMenu, MenuSettingsLevel1};

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum MenuType {
    MainCategories,
    CategorySettingList,
    SettingEdit,
}

// Marker component for the main settings menu UI
#[derive(Component)]
pub struct SettingsMenu {
    pub menu_type: MenuType,
    pub selected_item_idx: usize,
}

#[derive(Component)]
pub struct SCamera;

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, States, Default)]
pub enum SettingsState {
    /// Selects which Setting file/category to edit in the UI (Audio, Video, etc)
    #[default]
    Lv1ClassSelection,
    /// Lists the settings available in the file for later editing (Volume, Control Type, etc)
    Lv2List,
    /// Allows the user to select a new value for the setting (10% volume, 50% volume, etc)
    Lv3ValueEdit(MenuSettingsLevel1),
}

#[derive(Component)]
pub struct MenuItem {
    pub idx: usize,
    pub on_activate: MenuEvent,
}

impl MenuItem {
    pub fn new(idx: usize, on_activate: MenuEvent) -> Self {
        MenuItem { idx, on_activate }
    }
}

#[derive(Event, Debug, Clone, Copy, Default)]
pub enum MenuEvent {
    SaveAudioSetting(AudioSettingsValue),
    EditAudioSetting(AudioSettingsMenu),
    SettingClassSelected(MenuSettingsLevel1),
    Back(MenuEvBack),
    #[default]
    None,
}

impl MenuEvent {
    pub fn is_none(&self) -> bool {
        matches!(self, MenuEvent::None)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum AudioSettingsValue {
    volume_master(AudioLevel),
    volume_music(AudioLevel),
    volume_effects(AudioLevel),
    volume_ambient(AudioLevel),
    volume_voice_chat(AudioLevel),
    sound_output(SoundOutput),
    audio_positioning(AudioPositioning),
    feedback_delay(FeedbackDelay),
    feedback_eq(FeedbackEQ),
}

#[derive(Event, Debug, Clone, Copy)]
pub struct MenuEvBack;

#[derive(Event, Debug, Clone, Copy)]
pub struct MenuSettingClassSelected {
    pub menu: MenuSettingsLevel1,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct AudioSettingSelected {
    pub setting: AudioSettingsMenu,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SaveAudioSetting {
    pub value: AudioSettingsValue,
}
