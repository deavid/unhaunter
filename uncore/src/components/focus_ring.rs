use bevy::prelude::*;

/// A component for rendering focus ring visual effects around entities.
///
/// Focus rings appear as glowing vignette-style rings around entities such as ghosts
/// and breaches, creating a visual highlighting effect.
#[derive(Component, Debug, Clone)]
pub struct FocusRing {
    pub pulse_timer: f32,
}

impl Default for FocusRing {
    fn default() -> Self {
        Self { pulse_timer: 0.0 }
    }
}

impl FocusRing {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
