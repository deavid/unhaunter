use bevy::prelude::*;
use bevy::ecs::entity::{EntityMapper, MapEntities};
use serde::{Deserialize, Serialize};

/// Message sent by a client to report its owned entity state.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct ExportStateMessage {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub is_running: bool,
    pub frame: u16,
    pub is_hiding: bool,
    pub stamina: f32,
    pub health: f32,
    pub sanity: f32,
    pub is_spectating: bool,
}

/// Message sent by the server to grant ownership of an entity to a client.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct OwnershipGranted {
    pub entity: Entity,
}

impl MapEntities for OwnershipGranted {
    fn map_entities<M: EntityMapper>(&mut self, mapper: &mut M) {
        self.entity = mapper.get_mapped(self.entity);
    }
}

/// Message sent by the server to revoke ownership of an entity from a client.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct OwnershipReleased {
    pub entity: Entity,
}

impl MapEntities for OwnershipReleased {
    fn map_entities<M: EntityMapper>(&mut self, mapper: &mut M) {
        self.entity = mapper.get_mapped(self.entity);
    }
}
