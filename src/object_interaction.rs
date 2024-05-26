//! This module defines the `GameConfig` struct, which stores global parameters
//! for the game, including those related to the object-ghost interaction system.

use bevy::prelude::*;

/// Stores global game configuration parameters.
#[derive(Resource, Debug, Clone)]
pub struct ObjectInteractionConfig {
    /// The rate at which objects passively accumulate charge over time.
    pub object_charge_rate: f32,
    /// A multiplier for the discharge rate of Attractive objects when the ghost is nearby.
    pub attractive_discharge_multiplier: f32,
    /// A multiplier for the discharge rate of Repulsive objects when the ghost is nearby.
    pub repulsive_discharge_multiplier: f32,
    /// The radius around an object within which the ghost can influence its charge.
    pub object_discharge_radius: f32,
}

impl Default for ObjectInteractionConfig {
    fn default() -> Self {
        Self {
            object_charge_rate: 0.01,
            attractive_discharge_multiplier: 0.05,
            repulsive_discharge_multiplier: 0.02,
            object_discharge_radius: 3.0,
        }
    }
}
