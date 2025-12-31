use bevy::prelude::*;
use uncore_foundation::types::light::LightType;

// Define the LightSource component
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct LightSource {
    // Add fields for the LightSource component here
}

// Define the LightLevel component
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct LightLevel {
    pub lux: f32, // Represents current light level at player's position
}

/// Light emitter functionality (Public Interface).
#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct LightEmitter {
    pub power: f32,
    pub color: Color,
    pub light_type: LightType,
}
