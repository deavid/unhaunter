use bevy::prelude::*;
use uncore::components::board::position::Position;

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
    pub life: f32,
}
