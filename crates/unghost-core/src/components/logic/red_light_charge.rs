use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Runtime mode for ghosts that react to red light presence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect, Default)]
pub enum RedLightChargeMode {
    #[default]
    Charging,
    Discharging,
}

/// Charge/discharge controller for RL Presence ghost behavior.
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct GhostRedLightCharge {
    pub mode: RedLightChargeMode,
    pub charge: f32,
    pub threshold: f32,
    pub charge_rate: f32,
    pub discharge_rate: f32,
    pub red_react_threshold: f32,
}

impl Default for GhostRedLightCharge {
    fn default() -> Self {
        Self {
            mode: RedLightChargeMode::Charging,
            charge: 0.0,
            threshold: 1.0,
            charge_rate: 0.22,
            discharge_rate: 0.11,
            red_react_threshold: 0.2,
        }
    }
}
