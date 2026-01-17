use bevy::prelude::*;
use unsettings_core::controls::ControlKeys;
use unspatial_core::position::Position;

pub mod game_config;
pub use game_config::GameConfig;

/// Shared resource containing player state for cross-domain access.
#[derive(Resource, Debug, Clone)]
pub struct PlayerState {
    pub id: usize,
    pub health: f32,
    pub sanity: f32,
    pub position: Position,
    pub mean_sound: f32,
    pub controls: ControlKeys,
    pub hiding_spot: Option<Entity>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            id: 0,
            health: 100.0,
            sanity: 100.0,
            position: Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                visual_priority: 0.0,
            },
            mean_sound: 0.0,
            controls: ControlKeys::default(),
            hiding_spot: None,
        }
    }
}

/// Resource that acts as a virtual joystick for player movement.
/// All input systems write to this resource, and the movement system reads from it.
#[derive(Resource, Default)]
pub struct PlayerInput {
    /// The desired movement direction and magnitude.
    /// This is a normalized Vec2 where:
    /// - x represents left/right movement (-1 is left, 1 is right)
    /// - y represents up/down movement (-1 is down, 1 is up)
    ///
    /// If no movement is desired, this will be Vec2::ZERO
    pub movement: Vec2,

    /// Optional target position for click-to-move
    pub target_position: Option<Vec2>,
}

impl PlayerInput {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear any existing movement input
    pub fn clear(&mut self) {
        self.movement = Vec2::ZERO;
        self.target_position = None;
    }
}
