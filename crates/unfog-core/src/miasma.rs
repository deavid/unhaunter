use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use ndarray::Array3;

#[derive(Debug, Clone, Default, Resource)]
pub struct MiasmaGrid {
    pub pressure_field: Array3<f32>,
    pub velocity_field: Array3<Vec2>,
    pub room_modifiers: HashMap<String, f32>, // Room ID -> Modifier
}

impl MiasmaGrid {
    pub fn reset(&mut self) {
        self.pressure_field = Array3::default((0, 0, 0));
        self.velocity_field = Array3::default((0, 0, 0));
        self.room_modifiers.clear();
    }
}
