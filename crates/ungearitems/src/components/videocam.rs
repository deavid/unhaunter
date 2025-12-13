use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct Videocam {
    pub enabled: bool,
    pub display_glitch_timer: f32,
}


