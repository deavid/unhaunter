use bevy::prelude::*;
use unspatial_core::Direction;

/// Represents a piece of gear deployed in the game world.
#[derive(Component, Debug, Clone)]
pub struct DeployedGear {
    /// The direction the gear is facing.
    pub direction: Direction,
}
