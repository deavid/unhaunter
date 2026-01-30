use unfoundation_core::types::light::LightType;

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

    pub fn max(&self, other: &Self) -> Self {
        Self {
            visible: self.visible.max(other.visible),
            red: self.red.max(other.red),
            infrared: self.infrared.max(other.infrared),
            ultraviolet: self.ultraviolet.max(other.ultraviolet),
        }
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

    pub fn scale(&self, factor: f32) -> Self {
        Self {
            visible: self.visible * factor,
            red: self.red * factor,
            infrared: self.infrared * factor,
            ultraviolet: self.ultraviolet * factor,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LightFieldData {
    pub lux: f32,
    pub color: (f32, f32, f32),
    pub transmissivity: f32,
    pub additional: LightData,
}

impl Default for LightFieldData {
    fn default() -> Self {
        Self {
            lux: 0.0,
            color: (1.0, 1.0, 1.0),
            transmissivity: 1.0,
            additional: LightData::default(),
        }
    }
}
