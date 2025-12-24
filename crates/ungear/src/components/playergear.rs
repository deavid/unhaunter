use bevy::prelude::*;
use unplayer_core::components::HeldObject;

#[derive(Clone, Debug, Component, Default)]
pub struct PlayerGear {
    pub left_hand: Option<Entity>,
    pub right_hand: Option<Entity>,
    pub inventory: Vec<Entity>,
    pub held_item: Option<HeldObject>,
}
