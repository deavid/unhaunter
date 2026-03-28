pub mod evidence;
pub mod ghost;
pub mod resources;

use bevy::prelude::*;
use ghost::GhostType;

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct GhostSpawnRequest {
    pub ghost_types: Vec<GhostType>,
}
