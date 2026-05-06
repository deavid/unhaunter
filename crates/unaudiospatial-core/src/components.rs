use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct SpatialAudioInstance {
    pub sound_file: String,
    pub is_reverb: bool,
    pub initial_volume: f32,
    pub spawn_time: f32,
    pub spawn_frame: u32,
}

#[derive(Component, Debug, Clone)]
pub struct SpatialAudioFadeOut {
    pub timer: Timer,
}

impl SpatialAudioFadeOut {
    pub fn new(duration_secs: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }
}
