use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents the audio settings for the game.
#[derive(Resource, Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AudioSettings {
    /// The master volume level.
    pub volume_master: AudioLevel,
    /// The volume level for music.
    pub volume_music: AudioLevel,
    /// The volume level for sound effects.
    pub volume_effects: AudioLevel,
    /// The volume level for ambient sounds.
    pub volume_ambient: AudioLevel,
    /// The volume level for voice chat.
    pub volume_voice_chat: AudioLevel,
    /// The sound output mode (e.g., mono, headphones, speakers).
    pub sound_output: SoundOutput,
    /// The type of audio positioning.
    pub audio_positioning: AudioPositioning,
    /// The feedback delay setting.
    pub feedback_delay: FeedbackDelay,
    /// The feedback EQ setting.
    pub feedback_eq: FeedbackEQ,
}

/// Represents the different settings available for the audio
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum AudioSettingsValue {
    /// The master volume level.
    volume_master(AudioLevel),
    /// The volume level for music.
    volume_music(AudioLevel),
    /// The volume level for sound effects.
    volume_effects(AudioLevel),
    /// The volume level for ambient sounds.
    volume_ambient(AudioLevel),
    /// The volume level for voice chat.
    volume_voice_chat(AudioLevel),
    /// The sound output mode (e.g., mono, headphones, speakers).
    sound_output(SoundOutput),
    /// The type of audio positioning.
    audio_positioning(AudioPositioning),
    /// The feedback delay setting.
    feedback_delay(FeedbackDelay),
    /// The feedback EQ setting.
    feedback_eq(FeedbackEQ),
}

/// Represents the different volume levels.
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
    /// 0% volume.
    #[strum(to_string = "0%")]
    Vol000,
    /// 10% volume.
    #[strum(to_string = "10%")]
    Vol010,
    /// 20% volume.
    #[strum(to_string = "20%")]
    Vol020,
    /// 30% volume.
    #[strum(to_string = "30%")]
    Vol030,
    /// 40% volume.
    #[strum(to_string = "40%")]
    Vol040,
    /// 50% volume (default).
    #[default]
    #[strum(to_string = "50%")]
    Vol050,
    /// 60% volume.
    #[strum(to_string = "60%")]
    Vol060,
    /// 70% volume.
    #[strum(to_string = "70%")]
    Vol070,
    /// 80% volume.
    #[strum(to_string = "80%")]
    Vol080,
    /// 90% volume.
    #[strum(to_string = "90%")]
    Vol090,
    /// 100% volume.
    #[strum(to_string = "100%")]
    Vol100,
}

impl AudioLevel {
    /// Converts the `AudioLevel` to an `f32` volume multiplier.
    ///
    /// This uses a cubic curve to make the change in volume feel more natural.
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

/// Represents the different sound output modes.
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
    /// Mono sound output.
    Mono,
    /// Headphones sound output (default).
    #[default]
    Headphones,
    /// Speakers sound output.
    Speakers,
}
/// Represents the different audio positioning modes.
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
    /// Audio is positioned in screen space.
    ScreenSpace,
    /// Audio is positioned relative to the isometric view.
    Isometric,
    /// Audio is positioned relative to the character (default).
    #[default]
    CharacterRelative,
}

/// Represents the different feedback delay settings.
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
    /// No delay (0 microseconds).
    #[strum(to_string = "0us")]
    Delay0000us,
    /// 200 microseconds delay.
    #[strum(to_string = "200us")]
    Delay0200us,
    /// 300 microseconds delay (default).
    #[default]
    #[strum(to_string = "300us")]
    Delay0300us,
    /// 400 microseconds delay.
    #[strum(to_string = "400us")]
    Delay0400us,
}
/// Represents the feedback EQ setting.
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
    /// Enable feedback EQ (default).
    #[default]
    Yes,
    /// Disable feedback EQ.
    No,
}
