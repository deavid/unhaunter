use super::board::position::Position;
use bevy::prelude::*;

/// Component that marks an entity as wanting to move to a target position.
/// When this component is present on the player entity, the click-to-move pathing system
/// will attempt to direct the player toward this target.
#[derive(Component)]
pub struct MoveToTarget(pub Position);
