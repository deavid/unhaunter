//! This module defines the `GhostInfluence` component, which is used to determine
//! how an object in the game world influences the ghost's behavior.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents the type of influence an object has on the ghost.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum InfluenceType {
    /// The ghost is attracted to this object.
    Attractive,
    /// The ghost is repelled by this object.
    Repulsive,
}

/// This component stores the influence properties of a movable object, determining
/// how it affects the ghost's behavior.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GhostInfluence {
    /// The type of influence this object has on the ghost (`Attractive` or
    /// `Repulsive`).
    pub influence_type: InfluenceType,
    /// The current charge level of the object, ranging from 0.0 to 1.0. A higher
    /// charge value means a stronger influence on the ghost.
    pub charge_value: f32,
}
