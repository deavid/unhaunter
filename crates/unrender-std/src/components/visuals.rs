use bevy::prelude::*;
use unreplicon_core::network_id::NetworkId;

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

/// Component that stores the upscale factor of the asset (e.g., 3.0 for zoom03x).
/// Used to downscale the Transform so the object maintains its intended size.
#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct ResolutionFactor(pub f32);

impl Default for ResolutionFactor {
    fn default() -> Self {
        Self(1.0)
    }
}

impl ResolutionFactor {
    pub fn ratio(&self) -> f32 {
        1.0 / self.0
    }
}

/// Component that gives the renderer information about how to render the ghost implementation.
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
pub struct SpectralClarity {
    pub uv: f32,
    pub rl: f32,
    pub alpha: f32,
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

/// Component that identifies an entity as a light viewer (usually the player's eyes).
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct Viewer {
    pub id: NetworkId,
    pub health: f32,
    pub sanity: f32,
}

impl Default for Viewer {
    fn default() -> Self {
        Self {
            id: NetworkId(0),
            health: 100.0,
            sanity: 100.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum SpectralInfluenceType {
    #[default]
    Attractive,
    Repulsive,
}

/// Component for entities that have a "spectral signature" that reacts to non-visible light (UV/IR).
#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
pub struct SpectralInfluence {
    pub influence_type: SpectralInfluenceType,
    pub charge_value: f32,
    /// Persistence of UV light exposure (0.0 to 1.0+)
    pub uv_charge: f32,
    /// Persistence of Red light exposure (0.0 to 1.0+)
    pub red_charge: f32,
    /// Persistence of Infrared light exposure (0.0 to 1.0+)
    pub ir_charge: f32,
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
