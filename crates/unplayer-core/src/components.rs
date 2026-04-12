use bevy::ecs::entity::{EntityMapper, MapEntities};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use ungear_core::types::gear::equipment::Hand;
use unreplicon_core::network_id::NetworkId;
use uuid::Uuid;

/// Local component to tell which player entities is the current player. Camera follows this.
#[derive(Component, Debug, Clone, Default)]
pub struct MainPlayer;

/// Marks a player entity as an instance of the player class.
#[derive(Component, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component, Default)]
pub struct PlayerTag;

/// Component added to players who have disconnected but whose entity is being retained.
#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct PlayerDisconnected;

/// Marks a player entity that is connected but unresponsive (AFK for >2 minutes).
#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct PlayerInactive;

/// Component added to players who are spectating (dead or finished).
#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct PlayerSpectating;

#[derive(Component, Debug, Clone)]
pub struct InventoryNext {
    pub idx: Option<usize>,
}

impl InventoryNext {
    pub fn new(idx: usize) -> Self {
        Self { idx: Some(idx) }
    }

    pub fn non_empty() -> Self {
        Self { idx: Some(0) }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Inventory {
    pub hand: Hand,
}

impl Inventory {
    pub fn new_left() -> Self {
        Inventory { hand: Hand::Left }
    }

    pub fn new_right() -> Self {
        Inventory { hand: Hand::Right }
    }
}

#[derive(Component, Debug, Clone)]
pub struct InventoryStats {
    pub hand: Hand,
}

impl InventoryStats {
    pub fn left() -> Self {
        InventoryStats { hand: Hand::Left }
    }
    pub fn right() -> Self {
        InventoryStats { hand: Hand::Right }
    }
}

/// Represents a player character in the game world.
///
/// This component stores the player's identity and network identifiers.
#[derive(Component, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct PlayerSprite {
    /// The unique identifier for the player (persistent UUID).
    pub id: Uuid,
    /// The unique identifier for the player's Replicon client (u64).
    pub network_id: NetworkId,
}

impl MapEntities for PlayerSprite {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

impl Default for PlayerSprite {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            network_id: NetworkId(0),
        }
    }
}

impl PlayerSprite {
    /// Creates a new `PlayerSprite` with the specified identity.
    pub fn new(id: Uuid, network_id: NetworkId) -> Self {
        Self { id, network_id }
    }
}

/// Thin spawn request marker consumed by `unplayer-plugin` to materialize
/// player entities owned by the player domain.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct PlayerSpawnRequest {
    pub player_uuid: Uuid,
    pub network_id: NetworkId,
}

impl Default for PlayerSpawnRequest {
    fn default() -> Self {
        Self {
            player_uuid: Uuid::nil(),
            network_id: NetworkId(0),
        }
    }
}

/// Marks a player entity that is currently hiding.
#[derive(Component, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
#[component(map_entities)]
#[derive(Default)]
pub struct Hiding {
    pub hiding_spot: Option<Entity>,
}

impl bevy::ecs::entity::MapEntities for Hiding {
    fn map_entities<M: bevy::ecs::entity::EntityMapper>(&mut self, mapper: &mut M) {
        if let Some(ref mut h) = self.hiding_spot {
            *h = mapper.get_mapped(*h);
        }
    }
}
