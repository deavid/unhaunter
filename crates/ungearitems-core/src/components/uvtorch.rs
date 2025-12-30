use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct UVTorch {
    pub enabled: bool,
    pub output_power: f32,
}

impl Default for UVTorch {
    fn default() -> Self {
        Self {
            enabled: false,
            output_power: 0.0,
        }
    }
}
