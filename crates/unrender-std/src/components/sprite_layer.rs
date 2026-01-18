use bevy::prelude::*;

/// A component that represents a constant visual Z-offset for sorting/layering.
/// This is used to ensure certain entity types (like ghosts or players)
/// always appear above or below others within the same logical Z-coordinate.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct SpriteLayer(pub f32);
