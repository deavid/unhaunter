use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Component that uniquely identifies an entity across the network.
#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect, Default,
)]
#[reflect(Component)]
pub struct NetworkId(pub u64);

/// Component that marks an entity to be despawned after a few frames.
/// This is used to prevent panics when entities are despawned while still being referenced
/// by other systems or snapshots in the same frame.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
#[reflect(Component)]
pub struct ToBeDespawned {
    pub in_frames: usize,
}
