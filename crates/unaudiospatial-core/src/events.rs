use bevy::prelude::*;
use unspatial_core::position::Position;

#[derive(Message, Event, Debug, Clone)]
pub struct SoundEvent {
    pub sound_file: String,
    pub volume: f32,
    pub position: Option<Position>,
}
