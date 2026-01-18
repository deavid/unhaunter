use bevy::prelude::*;
/// Represents an object that is currently being held by the player.
#[derive(Component, Debug, Clone)]
pub struct HeldObject {
    pub entity: Entity,
}

#[derive(Clone, Debug, Component, Default)]
pub struct PlayerGear {
    pub left_hand: Option<Entity>,
    pub right_hand: Option<Entity>,
    pub inventory: Vec<Entity>,
    pub held_item: Option<HeldObject>,
}
