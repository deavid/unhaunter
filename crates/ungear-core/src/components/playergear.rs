use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents an object that is currently being held by the player.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct HeldObject {
    pub entity: Entity,
}

impl Default for HeldObject {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}

impl bevy::ecs::entity::MapEntities for HeldObject {
    fn map_entities<M: bevy::ecs::entity::EntityMapper>(&mut self, mapper: &mut M) {
        self.entity = mapper.get_mapped(self.entity);
    }
}

#[derive(Clone, Debug, Component, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct PlayerGear {
    pub left_hand: Option<Entity>,
    pub right_hand: Option<Entity>,
    pub inventory: Vec<Entity>,
    pub held_item: Option<HeldObject>,
}

impl bevy::ecs::entity::MapEntities for PlayerGear {
    fn map_entities<M: bevy::ecs::entity::EntityMapper>(&mut self, mapper: &mut M) {
        if let Some(ref mut h) = self.left_hand {
            *h = mapper.get_mapped(*h);
        }
        if let Some(ref mut h) = self.right_hand {
            *h = mapper.get_mapped(*h);
        }
        for h in self.inventory.iter_mut() {
            *h = mapper.get_mapped(*h);
        }
        if let Some(ref mut h) = self.held_item {
            h.map_entities(mapper);
        }
    }
}
