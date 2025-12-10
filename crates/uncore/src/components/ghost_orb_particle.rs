use crate::random_seed;
use bevy::prelude::*;
use rand::Rng; // Import the Rng trait
use std::f32::consts::TAU;

#[derive(Component, Debug)]
pub struct GhostOrbParticle {
    pub life: f32,
    pub initial_spawn_time: f32, // To calculate elapsed time for sine wave
    pub amplitude: Vec3,         // Max displacement for sine wave (X, Y, Z)
    pub frequency: Vec3,         // Speed of oscillation (X, Y, Z)
    pub phase: Vec3,             // Initial offset for sine wave (X, Y, Z)
    pub base_position: Vec3,     // The position around which it oscillates
}

impl GhostOrbParticle {
    pub fn new(life: f32, initial_spawn_time: f32, base_position: Vec3) -> Self {
        let mut rng = random_seed::rng();
        Self {
            life,
            initial_spawn_time,
            amplitude: Vec3::new(
                rng.random_range(0.5..1.0),
                rng.random_range(0.5..1.0),
                rng.random_range(0.05..0.1),
            ), // XY: +/-1.0, Z: +/-0.1
            frequency: Vec3::new(
                rng.random_range(0.2..0.5),
                rng.random_range(0.2..0.5),
                rng.random_range(0.3..0.6),
            ), // Slow oscillations
            phase: Vec3::new(
                rng.random_range(0.0..TAU),
                rng.random_range(0.0..TAU),
                rng.random_range(0.0..TAU),
            ),
            base_position,
        }
    }
}
