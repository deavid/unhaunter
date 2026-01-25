use bevy::prelude::*;

#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct RedTorch {
    pub enabled: bool,
    pub output_power: f32,
}
