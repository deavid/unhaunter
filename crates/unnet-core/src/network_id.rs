use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Component that uniquely identifies an entity across the network.
#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect, Default,
)]
#[reflect(Component)]
pub struct NetworkId(pub u64);
