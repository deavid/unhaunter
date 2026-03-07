use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents an object that is currently being held by the player.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
#[component(map_entities)]
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

/// Remaps the stored entity reference from one world's ID space to another.
///
/// This is required for replication correctness. When bevy_replicon deserializes
/// a [`HeldObject`] received from the server, the `entity` field contains a
/// **server-side** [`Entity`] ID. That ID is meaningless (or wrong) in the
/// client's world, where the same logical entity has a different ID.
///
/// The [`#[component(map_entities)]`](bevy::ecs::component::Component) attribute
/// on [`HeldObject`] causes Bevy's [`Component::map_entities`] method to delegate
/// to this implementation, which is called automatically by bevy_replicon's
/// `default_deserialize` after deserialization. The [`EntityMapper`] it receives
/// queries bevy_replicon's [`ServerEntityMap`](bevy_replicon::shared::server_entity_map::ServerEntityMap)
/// to translate the server ID to the corresponding client ID.
///
/// Without this, any system that reads `HeldObject::entity` on the client would
/// get a dangling or incorrect entity reference.
impl bevy::ecs::entity::MapEntities for HeldObject {
    fn map_entities<M: bevy::ecs::entity::EntityMapper>(&mut self, mapper: &mut M) {
        self.entity = mapper.get_mapped(self.entity);
    }
}

#[derive(Clone, Debug, Component, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
#[component(map_entities)]
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
