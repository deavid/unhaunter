use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Component that uniquely identifies an entity across the network.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkId(pub u64);

pub mod messages;
