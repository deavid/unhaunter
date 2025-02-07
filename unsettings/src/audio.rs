use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AudioSettings {
    pub volume_master: AudioLevel,
    pub volume_music: AudioLevel,
    pub volume_effects: AudioLevel,
    pub volume_ambient: AudioLevel,
    pub volume_voice_chat: AudioLevel,
    pub sound_output: SoundOutput,
    pub audio_positioning: AudioPositioning,
    pub feedback_delay: FeedbackDelay,
    pub feedback_eq: FeedbackEQ,
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Reflect,
    Component,
    strum::EnumIter,
    strum::Display,
)]
pub enum AudioLevel {
    #[strum(to_string = "0%")]
    Vol000,
    #[strum(to_string = "10%")]
    Vol010,
    #[strum(to_string = "20%")]
    Vol020,
    #[strum(to_string = "30%")]
    Vol030,
    #[strum(to_string = "40%")]
    Vol040,
    #[default]
    #[strum(to_string = "50%")]
    Vol050,
    #[strum(to_string = "60%")]
    Vol060,
    #[strum(to_string = "70%")]
    Vol070,
    #[strum(to_string = "80%")]
    Vol080,
    #[strum(to_string = "90%")]
    Vol090,
    #[strum(to_string = "100%")]
    Vol100,
}

impl AudioLevel {
    pub fn as_f32(&self) -> f32 {
        let v: f32 = match self {
            AudioLevel::Vol000 => 0.00,
            AudioLevel::Vol010 => 0.10,
            AudioLevel::Vol020 => 0.20,
            AudioLevel::Vol030 => 0.30,
            AudioLevel::Vol040 => 0.40,
            AudioLevel::Vol050 => 0.50,
            AudioLevel::Vol060 => 0.60,
            AudioLevel::Vol070 => 0.70,
            AudioLevel::Vol080 => 0.80,
            AudioLevel::Vol090 => 0.90,
            AudioLevel::Vol100 => 1.00,
        };

        v * v * v
    }
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Reflect,
    Component,
    strum::EnumIter,
    strum::Display,
)]
pub enum SoundOutput {
    Mono,
    #[default]
    Headphones,
    Speakers,
}
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Reflect,
    Component,
    strum::EnumIter,
    strum::Display,
)]
pub enum AudioPositioning {
    ScreenSpace,
    Isometric,
    #[default]
    CharacterRelative,
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Reflect,
    Component,
    strum::EnumIter,
    strum::Display,
)]
pub enum FeedbackDelay {
    #[strum(to_string = "0us")]
    Delay0000us,
    #[strum(to_string = "200us")]
    Delay0200us,
    #[default]
    #[strum(to_string = "300us")]
    Delay0300us,
    #[strum(to_string = "400us")]
    Delay0400us,
}
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Reflect,
    Component,
    strum::EnumIter,
    strum::Display,
)]
pub enum FeedbackEQ {
    #[default]
    Yes,
    No,
}
