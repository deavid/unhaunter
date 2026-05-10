use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCategory {
    Effects,
    VoiceChat,
    Master,
}

#[derive(Component, Debug, Clone)]
pub struct FlatAudio {
    pub sound_file: String,
    pub volume_multiplier: f32,
    pub category: AudioCategory,
}

#[derive(Component, Debug, Clone)]
pub struct SpatialAudioInstance {
    pub sound_file: String,
    pub is_reverb: bool,
    pub initial_volume: f32,
    pub spawn_time: f32,
    pub spawn_frame: u32,
    pub position: Option<unspatial_core::position::Position>,
}

#[derive(Component, Debug, Clone)]
pub struct SpatialAudioDelayedDespawn {
    pub timer: Timer,
}

impl SpatialAudioDelayedDespawn {
    pub fn new(duration_secs: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }
}
