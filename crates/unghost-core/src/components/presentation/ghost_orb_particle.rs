use bevy::prelude::*;
use rand::prelude::*;
use std::f32::consts::TAU;
use uncommon_app_core::random_seed;

/// Local-only particle component for ghost orb visual effects.
/// Never replicated. Owned and ticked entirely by unghost-presentation.
#[derive(Component, Debug)]
pub struct GhostOrbParticle {
    pub life: f32,
    pub initial_spawn_time: f32,
    pub amplitude: Vec3,
    pub frequency: Vec3,
    pub phase: Vec3,
    pub base_position: Vec3,
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
            ),
            frequency: Vec3::new(
                rng.random_range(0.2..0.5),
                rng.random_range(0.2..0.5),
                rng.random_range(0.3..0.6),
            ),
            phase: Vec3::new(
                rng.random_range(0.0..TAU),
                rng.random_range(0.0..TAU),
                rng.random_range(0.0..TAU),
            ),
            base_position,
        }
    }
}
