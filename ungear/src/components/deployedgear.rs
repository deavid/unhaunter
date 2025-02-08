use crate::types::gear::Gear;
use bevy::prelude::*;
use uncore::components::board::direction::Direction;

/// Represents a piece of gear deployed in the game world.
#[derive(Component, Debug, Clone)]
pub struct DeployedGear {
    /// The direction the gear is facing.
    pub direction: Direction,
}

/// Component to store the GearKind of a deployed gear entity.
#[derive(Component, Debug, Clone)]
pub struct DeployedGearData {
    pub gear: Gear,
}
