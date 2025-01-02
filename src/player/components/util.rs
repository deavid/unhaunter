use bevy::prelude::*;

/// Represents an object that is currently being held by the player.
#[derive(Component, Debug, Clone)]
pub struct HeldObject {
    pub entity: Entity,
}

/// Marks a player entity that is currently hiding.
#[derive(Component)]
pub struct Hiding {
    pub hiding_spot: Entity,
}
