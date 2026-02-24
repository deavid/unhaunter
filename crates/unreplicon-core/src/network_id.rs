use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A 64-bit identifier that is unique per entity across the network.
///
/// Zero is reserved as a sentinel for the host / listen-server player.
#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect, Default,
)]
#[reflect(Component)]
pub struct NetworkId(pub u64);

/// Marks an entity as pending despawn after `in_frames` update ticks.
///
/// Used to keep despawn events alive long enough for all clients to receive them
/// before the entity is removed from the ECS world.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
#[reflect(Component)]
pub struct ToBeDespawned {
    pub in_frames: usize,
}
