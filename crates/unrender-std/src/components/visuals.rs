use bevy::prelude::*;

/// Component that controls how an entity reacts to general lighting and exposure.
#[derive(Component, Debug, Clone, Copy)]
pub struct LightSensitive {
    /// Multiplier for the calculated relative exposure (default: 1.0).
    pub exposure_factor: f32,
    /// Constant bias added to exposure (default: 0.0).
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

/// Component for entities that react to Ultraviolet light (evidence/fluorescence).
#[derive(Component, Debug, Clone, Copy)]
pub struct UltravioletSensitive {
    /// How much the UV light affects the brightness (fluorescence).
    pub intensity: f32,
    /// How much the UV light shifts the color towards a specific tint.
    pub color_shift: f32,
}

impl Default for UltravioletSensitive {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            color_shift: 0.0,
        }
    }
}

/// Component for entities that react to Infrared light (Ghost Orbs, NVG).
#[derive(Component, Debug, Clone, Copy)]
pub struct InfraredSensitive {
    /// Multiplier for infrared light reception.
    pub intensity: f32,
    /// If true, the entity is only visible via infrared/NVG.
    pub thresholds: Option<f32>,
}

impl Default for InfraredSensitive {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            thresholds: None,
        }
    }
}

/// Component for entities that should oscillate or flicker their alpha/brightness.
#[derive(Component, Debug, Clone, Copy)]
pub struct AlphaModulator {
    /// Speed of oscillation.
    pub frequency: f32,
    /// Depth of oscillation (0.0 to 1.0).
    pub amplitude: f32,
}

impl Default for AlphaModulator {
    fn default() -> Self {
        Self {
            frequency: 1.0,
            amplitude: 0.5,
        }
    }
}

/// Specialized component for ghosts/breaches that use complex logic.
/// This acts as a transitional component to keep the complex math out of the main loop
/// while still removing the hardcoded SpriteType checks.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct EctoplasmVisuals {
    /// Whether to use the cube-root visibility curve.
    pub use_breach_curve: bool,
}
