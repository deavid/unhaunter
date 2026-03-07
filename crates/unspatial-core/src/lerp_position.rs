use crate::position::Position;
use bevy::prelude::*;

/// Visual interpolation buffer. Present on remote entities (ghost, remote players).
/// Absent on the local player — direct Position→Transform at zero latency.
///
/// Each frame, `advance_lerp_position` converges `current` toward the entity's `Position`.
/// `apply_perspective` reads `current` instead of `Position` when this component is present.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct LerpPosition {
    pub current: Position,
    pub speed: f32,
}

impl LerpPosition {
    pub fn new(initial: Position) -> Self {
        Self {
            current: initial,
            speed: 15.0,
        }
    }
    pub fn with_speed(initial: Position, speed: f32) -> Self {
        Self {
            current: initial,
            speed,
        }
    }
}
