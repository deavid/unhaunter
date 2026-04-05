use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents the type of influence an object has on the ghost.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize, Hash)]
pub enum InfluenceType {
    /// The ghost is attracted to this object.
    Attractive,
    /// The ghost is repelled by this object.
    Repulsive,
}

/// This component stores the influence properties of a movable object, determining
/// how it affects the ghost's behavior.
#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct GhostInfluence {
    /// The type of influence this object has on the ghost (`Attractive` or `Repulsive`).
    pub influence_type: InfluenceType,
    /// The current charge level of the object, ranging from 0.0 to 1.0.
    pub charge_value: f32,
}

impl Default for GhostInfluence {
    fn default() -> Self {
        Self {
            influence_type: InfluenceType::Attractive,
            charge_value: 0.0,
        }
    }
}
