use bevy::prelude::*;

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
/// while still removing the hardcoded visual checks.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct EctoplasmVisuals {
    /// Whether to use the cube-root visibility curve.
    pub use_breach_curve: bool,
}

/// Component for entities that should emit their own light (e.g., repellent particles, ghost orbs).
/// This follows a PBR-like approach where MapColor is the Albedo (multiplicative)
/// and Emissive is the additive glow.
#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct Emissive {
    /// Additive color for the glow.
    pub color: Color,
    /// Constant intensity of the glow.
    pub intensity: f32,
    /// Light reactivity (fluorescence/phosphorescence): how much environment lighting stimulates this emission.
    pub light_reactivity: f32,
    /// Speed of the pulse oscillation (0.0 to disable).
    pub pulse_speed: f32,
}

impl Default for Emissive {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 0.0,
            light_reactivity: 0.0,
            pulse_speed: 0.0,
        }
    }
}

/// Component for entities that should flicker or distort (e.g., ghosts).
#[derive(Component, Debug, Clone, Copy)]
pub struct Ethereal {
    /// Speed or intensity of alpha oscillation.
    pub alpha_oscillation: f32,
    /// Warping intensity for visual distortion.
    pub warp: f32,
    /// Stability factor (visual clarity).
    pub stability: f32,
    /// True if the entity is in a "warning" state (e.g., hunt warning).
    pub warning_active: bool,
    /// Intensity of the warning effect (0.0 to 1.0).
    pub warning_intensity: f32,
    /// True if the entity is currently targeting something.
    pub hunt_target: bool,
    /// Recovery or calm time remaining.
    pub calm_time_secs: f32,
    /// Visual impact from being "hit" by something (e.g., repellent).
    pub hit_delta: f32,
    /// Visual impact from a "miss" check.
    pub miss_delta: f32,
}

impl Default for Ethereal {
    fn default() -> Self {
        Self {
            alpha_oscillation: 1.0,
            warp: 0.0,
            stability: 1.0,
            warning_active: false,
            warning_intensity: 0.0,
            hunt_target: false,
            calm_time_secs: 0.0,
            hit_delta: 0.0,
            miss_delta: 0.0,
        }
    }
}

/// Component for solid entities that should cast shadows or have standard responsive lighting.
#[derive(Component, Debug, Clone, Copy)]
pub struct ShadowCaster {
    /// Strength of the shadow cast by this entity.
    pub shadow_strength: f32,
}

impl Default for ShadowCaster {
    fn default() -> Self {
        Self {
            shadow_strength: 1.0,
        }
    }
}
