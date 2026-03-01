use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A 64-bit identifier that is unique per entity across the network.
///
/// Zero is reserved as a sentinel for the host / listen-server player.
#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect, Default,
)]
#[reflect(Component)]
pub struct NetworkId(pub u64);

impl From<Uuid> for NetworkId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid.to_u128_le() as u64)
    }
}
