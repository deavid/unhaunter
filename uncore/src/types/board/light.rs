/// Represents different types of light in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    /// Standard visible light.
    Visible,
    /// Red light, often used for night vision or specific ghost interactions.
    Red,
    /// Infrared light used for night vision cameras.
    InfraRedNV,
    /// Ultraviolet light, used to reveal evidence or trigger ghost reactions.
    UltraViolet,
}

/// Stores the intensity of different light types at a specific location.
///
/// This data structure is used to represent the combined light levels from various
/// sources, such as ambient light, flashlights, and ghost effects.
#[derive(Debug, Clone, Copy, Default)]
pub struct LightData {
    /// Intensity of visible light.
    pub visible: f32,
    /// Intensity of red light.
    pub red: f32,
    /// Intensity of infrared light.
    pub infrared: f32,
    /// Intensity of ultraviolet light.
    pub ultraviolet: f32,
}

impl LightData {
    pub const UNIT_VISIBLE: Self = Self {
        visible: 1.0,
        red: 0.0,
        infrared: 0.0,
        ultraviolet: 0.0,
    };

    pub fn from_type(light_type: LightType, strength: f32) -> Self {
        match light_type {
            LightType::Visible => Self {
                visible: strength,
                ..Default::default()
            },
            LightType::Red => Self {
                red: strength,
                ..Default::default()
            },
            LightType::InfraRedNV => Self {
                infrared: strength,
                ..Default::default()
            },
            LightType::UltraViolet => Self {
                ultraviolet: strength,
                ..Default::default()
            },
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            visible: self.visible + other.visible,
            red: self.red + other.red,
            infrared: self.infrared + other.infrared,
            ultraviolet: self.ultraviolet + other.ultraviolet,
        }
    }

    pub fn magnitude(&self) -> f32 {
        let sq_m = self.visible.powi(2)
            + self.red.powi(2)
            + self.infrared.powi(2)
            + self.ultraviolet.powi(2);
        sq_m.sqrt()
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude() + 1.0;
        Self {
            visible: self.visible / mag,
            red: self.red / mag,
            infrared: self.infrared / mag,
            ultraviolet: self.ultraviolet / mag,
        }
    }
}
