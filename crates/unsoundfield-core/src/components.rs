use bevy::prelude::*;

/// A component that emits sound into the environment field (simulation).
///
/// This is used by the sound field simulation system in `unsoundfield-plugin` to
/// update the sound grid of the board.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SoundFieldSource {
    /// The volume/intensity of the sound being emitted into the field.
    pub volume: f32,
}
