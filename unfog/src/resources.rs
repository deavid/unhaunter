use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct MiasmaConfig {
    pub initial_room_pressure: f32,
    pub initial_outside_pressure: f32,
    pub miasma_visibility_factor: f32,
    pub diffusion_rate: f32,
}

impl Default for MiasmaConfig {
    fn default() -> Self {
        Self {
            initial_room_pressure: 0.3,
            initial_outside_pressure: 0.0,
            miasma_visibility_factor: 0.5,
            diffusion_rate: 5.0,
        }
    }
}
