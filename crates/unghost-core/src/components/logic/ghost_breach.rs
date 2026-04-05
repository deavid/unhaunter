use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Entity-class marker for the ghost's breach entity (the portal/spawn point).
/// This component is replicated so all clients know the breach exists and where it is.
#[derive(Component, Debug, Default, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct GhostBreach;
