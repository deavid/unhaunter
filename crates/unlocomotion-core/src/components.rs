use bevy::ecs::entity::{EntityMapper, MapEntities};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use unspatial_core::direction::Direction;

/// Component for managing player locomotion state.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct PlayerLocomotionState {
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
            movement: Direction::zero(),
            velocity: Vec2::ZERO,
        }
    }
}
