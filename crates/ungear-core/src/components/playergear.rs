use bevy::prelude::*;
use bevy::ecs::entity::{EntityMapper, MapEntities};
use bevy::ecs::reflect::ReflectMapEntities;
use serde::{Deserialize, Serialize};

/// Represents an object that is currently being held by the player.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component, MapEntities)]
pub struct HeldObject {
    pub entity: Entity,
}

#[derive(Clone, Debug, Component, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default, MapEntities)]
pub struct PlayerGear {
    pub left_hand: Option<Entity>,
    pub right_hand: Option<Entity>,
    pub inventory: Vec<Entity>,
    pub held_item: Option<HeldObject>,
}

impl MapEntities for HeldObject {
    fn map_entities<M: EntityMapper>(&mut self, mapper: &mut M) {
        self.entity = mapper.get_mapped(self.entity);
    }
}

impl MapEntities for PlayerGear {
    fn map_entities<M: EntityMapper>(&mut self, mapper: &mut M) {
        if let Some(ref mut e) = self.left_hand { *e = mapper.get_mapped(*e); }
        if let Some(ref mut e) = self.right_hand { *e = mapper.get_mapped(*e); }
        for e in self.inventory.iter_mut() { *e = mapper.get_mapped(*e); }
        if let Some(ref mut ho) = self.held_item { ho.map_entities(mapper); }
    }
}
