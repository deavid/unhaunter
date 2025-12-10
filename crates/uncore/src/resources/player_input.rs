use bevy::prelude::*;

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
