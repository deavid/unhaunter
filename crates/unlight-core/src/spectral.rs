use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default, Serialize, Deserialize)]
pub enum SpectralInfluenceType {
    #[default]
    Attractive,
    Repulsive,
}

/// Component for entities that have a "spectral signature" that reacts to non-visible light (UV/IR).
#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct SpectralInfluence {
    pub influence_type: SpectralInfluenceType,
    pub charge_value: f32,
    pub uv_charge: f32,
    pub red_charge: f32,
    pub ir_charge: f32,
    pub uv_intensity: f32,
    pub uv_color_shift: f32,
    pub ir_intensity: f32,
    pub ir_threshold: Option<f32>,
}

impl SpectralInfluence {
    pub fn with_ultraviolet(mut self, intensity: f32, color_shift: f32) -> Self {
        self.uv_intensity = intensity;
        self.uv_color_shift = color_shift;
        self
    }

    pub fn with_infrared(mut self, intensity: f32, threshold: Option<f32>) -> Self {
        self.ir_intensity = intensity;
        self.ir_threshold = threshold;
        self
    }

    pub fn has_visual_charge(&self) -> bool {
        self.charge_value.abs() > f32::EPSILON
    }

    pub fn is_uv_sensitive(&self) -> bool {
        self.uv_intensity > 0.0
    }

    pub fn is_ir_sensitive(&self) -> bool {
        self.ir_intensity > 0.0
    }
}
