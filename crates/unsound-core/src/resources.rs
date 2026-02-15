use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use unspatial_core::boardposition::BoardPosition;

#[derive(Clone, Debug, Resource, Default)]
pub struct SoundGrid {
    pub sound_field: HashMap<BoardPosition, Vec<Vec2>>,
}

impl SoundGrid {
    pub fn reset(&mut self) {
        self.sound_field.clear();
    }
}
