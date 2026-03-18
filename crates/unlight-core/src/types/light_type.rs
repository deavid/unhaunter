use bevy::prelude::*;

/// Represents different types of light in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum LightType {
    /// Standard visible light.
    #[default]
    Visible,
    /// Red light, often used for night vision or specific ghost interactions.
    Red,
    /// Infrared light used for night vision cameras.
    InfraRedNV,
    /// Ultraviolet light, used to reveal evidence or trigger ghost reactions.
    UltraViolet,
}
