use bevy::prelude::*;
use enum_iterator::Sequence;
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence)]
pub enum AudioLevel {
    Vol000,
    Vol010,
    Vol020,
    Vol030,
    Vol040,
    #[default]
    Vol050,
    Vol060,
    Vol070,
    Vol080,
    Vol090,
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence)]
pub enum SoundOutput {
    Mono,
    #[default]
    Headphones,
    Speakers,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence)]
pub enum AudioPositioning {
    ScreenSpace,
    Isometric,
    #[default]
    CharacterRelative,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence)]
pub enum FeedbackDelay {
    Delay0000us,
    Delay0200us,
    #[default]
    Delay0300us,
    Delay0400us,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence)]
pub enum FeedbackEQ {
    #[default]
    Yes,
    No,
}
