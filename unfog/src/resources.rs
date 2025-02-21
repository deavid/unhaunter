use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct MiasmaConfig {
    pub initial_room_pressure: f32,
    pub initial_outside_pressure: f32,
    pub miasma_visibility_factor: f32,
    pub diffusion_rate: f32,
    pub velocity_scale: f32,
    pub inertia_factor: f32,
    pub friction: f32,
}

impl Default for MiasmaConfig {
    fn default() -> Self {
        Self {
            initial_room_pressure: 100.0,
            initial_outside_pressure: 0.0,
            miasma_visibility_factor: 0.18,
            diffusion_rate: 2.0,
            velocity_scale: 10.0,
            inertia_factor: 10.0,
            friction: 0.1,
        }
    }
}
