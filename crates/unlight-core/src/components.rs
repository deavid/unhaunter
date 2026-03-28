use crate::types::light_type::LightType;
use bevy::prelude::*;

/// Light source component for entities that emit light
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct LightSource;

/// Light level at a specific location
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct LightLevel {
    pub lux: f32,
}

/// Light emitter functionality
#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct LightEmitter {
    pub power: f32,
    pub color: Color,
    pub light_type: LightType,
}

/// Component that controls how an entity reacts to general lighting and exposure
#[derive(Component, Debug, Clone, Copy)]
pub struct LightSensitive {
    pub exposure_factor: f32,
    pub bias: f32,
}

impl Default for LightSensitive {
    fn default() -> Self {
        Self {
            exposure_factor: 1.0,
            bias: 0.0,
        }
    }
}
