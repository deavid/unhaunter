use bevy::prelude::*;
use bevy_persistent::Persistent;
use strum::IntoEnumIterator;
use unsettings::audio::{AudioLevel, AudioSettings};

use crate::components::{AudioValueVariant, MenuEvent};

#[derive(strum::Display, strum::EnumIter, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MenuSettingsLevel1 {
    Gameplay,
    Video,
    Audio,
    Profile,
}

impl MenuSettingsLevel1 {
    pub fn menu_event(&self) -> MenuEvent {
        use MenuSettingsLevel1 as m;
        match self {
            MenuSettingsLevel1::Gameplay => MenuEvent::SettingClassSelected(m::Gameplay),
            MenuSettingsLevel1::Video => MenuEvent::SettingClassSelected(m::Video),
            MenuSettingsLevel1::Audio => MenuEvent::SettingClassSelected(m::Audio),
            // We disable Profile for now
            MenuSettingsLevel1::Profile => MenuEvent::None,
        }
    }

    pub fn iter_events() -> Vec<(String, MenuEvent)> {
        use strum::IntoEnumIterator;
        Self::iter()
            .map(|s| (s.to_string(), s.menu_event()))
            .collect::<Vec<_>>()
    }
}

#[derive(strum::Display, strum::EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioSettingsMenu {
    #[strum(to_string = "Master Volume")]
    VolumeMaster,
    #[strum(to_string = "Music Volume")]
    VolumeMusic,
    #[strum(to_string = "Effects Volume")]
    VolumeEffects,
    #[strum(to_string = "Ambient Volume")]
    VolumeAmbient,
    #[strum(to_string = "VoiceChat Volume")]
    VolumeVoiceChat,
    #[strum(to_string = "Sound Output")]
    SoundOutput,
    #[strum(to_string = "Audio Positioning")]
    AudioPositioning,
    #[strum(to_string = "Feedback Delay")]
    FeedbackDelay,
    #[strum(to_string = "Feedback EQ")]
    FeedbackEq,
}

impl AudioSettingsMenu {
    pub fn menu_event(&self) -> MenuEvent {
        #[allow(clippy::match_single_binding)]
        match self {
            // <-- add here the events for specific menus
            Self::VolumeMusic => MenuEvent::EditAudioSetting(*self),
            _ => MenuEvent::None,
        }
    }

    pub fn setting_value(&self, audio_settings: &Res<Persistent<AudioSettings>>) -> String {
        match self {
            AudioSettingsMenu::VolumeMaster => audio_settings.volume_master.to_string(),
            AudioSettingsMenu::VolumeMusic => audio_settings.volume_music.to_string(),
            AudioSettingsMenu::VolumeEffects => audio_settings.volume_effects.to_string(),
            AudioSettingsMenu::VolumeAmbient => audio_settings.volume_ambient.to_string(),
            AudioSettingsMenu::VolumeVoiceChat => audio_settings.volume_voice_chat.to_string(),
            AudioSettingsMenu::SoundOutput => audio_settings.sound_output.to_string(),
            AudioSettingsMenu::AudioPositioning => audio_settings.audio_positioning.to_string(),
            AudioSettingsMenu::FeedbackDelay => audio_settings.feedback_delay.to_string(),
            AudioSettingsMenu::FeedbackEq => audio_settings.feedback_eq.to_string(),
        }
    }

    pub fn iter_events_item(&self) -> Vec<(String, MenuEvent)> {
        match self {
            AudioSettingsMenu::VolumeMusic => AudioLevel::iter()
                .map(|s| {
                    (
                        s.to_string(),
                        MenuEvent::SaveAudioSetting(*self, AudioValueVariant::AudioLevel(s)),
                    )
                })
                .collect::<Vec<_>>(),
            _ => vec![],
        }
    }

    pub fn iter_events(
        audio_settings: &Res<Persistent<AudioSettings>>,
    ) -> Vec<(String, MenuEvent)> {
        use strum::IntoEnumIterator;
        Self::iter()
            .map(|s| {
                (
                    format!("{}: {}", s, s.setting_value(audio_settings)),
                    s.menu_event(),
                )
            })
            .collect::<Vec<_>>()
    }
}

/// Common type for setting the value of the options
pub enum OptionValue {
    List(OptionList),
    Text(String),
}

pub trait OptionListTrait {
    fn list_options(&self) -> Vec<String>;
}

pub enum OptionValue2 {
    List(Box<dyn OptionListTrait>),
    Text(String),
}

pub struct OptionList {
    pub display_values: Vec<String>,
    pub selected_idx: usize,
}
