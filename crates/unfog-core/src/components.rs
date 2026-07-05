use bevy::prelude::*;
use unspatial_core::position::Position;

#[derive(Component, Debug, Clone)]
pub struct MiasmaSprite {
    /// Original position.
    pub base_position: Position,
    /// Radius of the circular motion.
    pub radius: f32,
    /// Speed of the circular motion.
    pub angular_speed: f32,
    /// Initial phase offset for circular motion.
    pub phase: f32,
    /// X offset for Perlin noise sampling
    pub noise_offset_x: f32,
    /// Y offset for Perlin noise sampling
    pub noise_offset_y: f32,
    /// How visible is this sprite of Miasma
    pub visibility: f32,
    /// How long has been visible
    pub time_alive: f32,
    /// Trigger for despawning
    pub despawn: bool,
    /// Despawning metric, when reaches zero despawns.
    pub life: f32,
    /// Velocity response to flow.
    pub vel_speed: f32,
    /// Speed of movement of the particle so it denoises the miasma velocity field.
    pub direction: Vec2,
    /// Base visual scale of the sprite.
    pub base_scale: f32,
}

/// A glowing red hazard that drifts toward players when miasma pressure is high.
#[derive(Component, Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct MiasmaHazardParticle {
    /// Velocity of the particle.
    pub velocity: Vec2,
    /// Time in seconds since the particle was spawned.
    pub time_alive: f32,
}

impl Default for MiasmaHazardParticle {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            time_alive: 0.0,
        }
    }
}
