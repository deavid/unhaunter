use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Component that gives the renderer information about how to render the ghost implementation.
#[derive(Component, Debug, Clone, Copy, Reflect, Default, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct SpectralClarity {
    pub uv: f32,
    pub rl: f32,
    pub alpha: f32,
}
