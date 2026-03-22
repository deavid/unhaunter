use bevy::ecs::entity::{EntityMapper, MapEntities};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use ungear_core::types::gear::equipment::Hand;
use unreplicon_core::network_id::NetworkId;
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;
use uuid::Uuid;

#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct MainPlayer;

impl MapEntities for MainPlayer {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

/// Component added to players who have disconnected but whose entity is being retained.
#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct PlayerDisconnected;

/// Marks a player entity that is connected but unresponsive (no heartbeat for >5s).
#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct PlayerInactive;

/// Component added to players who are spectating (dead or finished).
#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct PlayerSpectating;

impl MapEntities for PlayerSpectating {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

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

/// Component for managing player locomotion state.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct PlayerLocomotionState {
    /// The player's initial spawn position when the level started.
    pub spawn_position: Position,
    /// The player's movement direction based on WASD controls.
    pub movement: Direction,
    /// The current normalized input direction (raw velocity), used for animation.
    /// Zero when the player is not moving, unit vector when moving.
    #[serde(default)]
    pub velocity: Vec2,
}

impl MapEntities for PlayerLocomotionState {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

impl Default for PlayerLocomotionState {
    fn default() -> Self {
        Self {
            spawn_position: Position::default(),
            movement: Direction::zero(),
            velocity: Vec2::ZERO,
        }
    }
}

impl PlayerSprite {
    /// Creates a new `PlayerSprite` with the specified identity.
    pub fn new(id: Uuid, network_id: NetworkId) -> Self {
        Self { id, network_id }
    }
}

impl PlayerLocomotionState {
    pub fn new(spawn_position: Position) -> Self {
        Self {
            spawn_position,
            ..default()
        }
    }
}

/// Marks a player entity that is currently hiding.
#[derive(Component, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
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
