use bevy::prelude::*;
use unspatial_core::position::Position;

#[derive(Message, Event, Debug, Clone)]
pub struct SoundEvent {
    pub sound_file: String,
    pub volume: f32,
    pub position: Option<Position>,
}

#[derive(Message, Debug, Clone)]
pub struct LocalSoundEvent {
    pub sound_file: String,
    pub volume: f32,
    pub position: Option<Position>,
}

impl From<&LocalSoundEvent> for SoundEvent {
    fn from(value: &LocalSoundEvent) -> Self {
        Self {
            sound_file: value.sound_file.clone(),
            volume: value.volume,
            position: value.position,
        }
    }
}
