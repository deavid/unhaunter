//! Player state resource for decoupling player data access.

use bevy::prelude::*;
use unsettings::controls::ControlKeys;

/// Shared resource containing player state for cross-domain access.
#[derive(Resource, Debug, Clone)]
pub struct PlayerState {
    pub id: usize,
    pub health: f32,
    pub sanity: f32,
    pub position: unspatial::Position,
    pub mean_sound: f32,
    pub controls: ControlKeys,
    pub hiding_spot: Option<bevy::prelude::Entity>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            id: 0,
            health: 100.0,
            sanity: 100.0,
            position: unspatial::Position { x: 0.0, y: 0.0, z: 0.0, global_z: 0.0 },
            mean_sound: 0.0,
            controls: ControlKeys::default(),
            hiding_spot: None,
        }
    }
}