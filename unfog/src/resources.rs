use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct MiasmaConfig {
    pub initial_room_pressure: f32,
    pub initial_outside_pressure: f32,
    pub max_miasma_alpha: f32,
}

impl Default for MiasmaConfig {
    fn default() -> Self {
        Self {
            initial_room_pressure: 0.8,
            initial_outside_pressure: 0.0,
            max_miasma_alpha: 0.5,
        }
    }
}
