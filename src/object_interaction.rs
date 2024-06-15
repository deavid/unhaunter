//! This module defines the `GameConfig` struct, which stores global parameters
//! for the game, including those related to the object-ghost interaction system.

use bevy::prelude::*;

/// Stores global configuration parameters for the object-ghost interaction system.
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
    /// A multiplier for the influence of Attractive objects on the ghost's destination score.
    pub attractive_influence_multiplier: f32,
    /// A multiplier for the influence of Repulsive objects on the ghost's destination score.
    pub repulsive_influence_multiplier: f32,
    // When the ghost decides to move, sample N possible choices to get the best one.
    pub num_destination_points_to_sample: usize,
}

impl Default for ObjectInteractionConfig {
    fn default() -> Self {
        Self {
            object_charge_rate: 0.01,
            attractive_discharge_multiplier: 0.05,
            repulsive_discharge_multiplier: 0.02,
            object_discharge_radius: 3.0,
            attractive_influence_multiplier: 1.0,
            repulsive_influence_multiplier: 1.0,
            num_destination_points_to_sample: 10,
        }
    }
}
