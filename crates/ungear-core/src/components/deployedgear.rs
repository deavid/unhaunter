use bevy::prelude::*;
use unspatial_core::direction::Direction;

/// Represents a piece of gear deployed in the game world.
#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize, Reflect)]
#[reflect(Component)]
pub struct DeployedGear {
    /// The direction the gear is facing.
    pub direction: Direction,
}
