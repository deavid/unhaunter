use super::board::position::Position;
use bevy::prelude::*;

/// Component that marks an entity as wanting to move to a target position.
/// When this component is present on the player entity, the click-to-move pathing system
/// will attempt to direct the player toward this target.
/// If interaction fields are present, the entity will attempt to interact with the target
/// entity after reaching the position.
#[derive(Component)]
pub struct MoveToTarget {
    pub position: Position,
    /// Optional entity to interact with after reaching the target position
    pub interaction_target: Option<Entity>,
}

impl MoveToTarget {
    /// Create a simple movement target without interaction
    pub fn new(position: Position) -> Self {
        Self {
            position,
            interaction_target: None,
        }
    }

    /// Create a movement target with an interaction after reaching the position
    pub fn with_interaction(position: Position, interaction_target: Entity) -> Self {
        Self {
            position,
            interaction_target: Some(interaction_target),
        }
    }
}
