use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct Videocam {
    pub output_power: f32,
}
