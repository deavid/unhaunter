use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Identifies a network participant. Mirrors Replicon's ClientId but
/// lives here to avoid a dependency on bevy_replicon in core crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum OwnerId {
    Client(Entity),
    Server,
}

impl OwnerId {
    pub fn is_server(&self) -> bool {
        matches!(self, OwnerId::Server)
    }
    pub fn is_client(&self) -> bool {
        matches!(self, OwnerId::Client(_))
    }
}

/// Marks which client is the simulation authority for this entity.
/// Inserted by the server. Replicated.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Owner(pub OwnerId);

/// Marker inserted by the client on entities it directly simulates.
/// Never replicated. Client-local only.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct LocallyOwned;
