use crate::components::logic::ghost_sprite::GhostBehaviorDynamics;
use bevy::prelude::*;
use bevy_platform::collections::HashSet;
use uninvestigation_core::evidence::Evidence;
use unspatial_core::position::Position;

/// Represents the status of the current haunting (narrative/gameplay).
#[derive(Clone, Debug, Resource)]
pub struct HauntState {
    /// Evidences of the current ghost
    pub evidences: HashSet<Evidence>,
    pub ghost_dynamics: GhostBehaviorDynamics,

    // Ghost warning state
    /// Current warning intensity (0.0-1.0)
    pub ghost_warning_intensity: f32,
    /// Source position of warning
    pub ghost_warning_position: Option<Position>,
    /// Position of the ghost breach
    pub breach_pos: Position,
}

impl Default for HauntState {
    fn default() -> Self {
        HauntState {
            evidences: Default::default(),
            ghost_dynamics: Default::default(),
            ghost_warning_intensity: 0.0,
            ghost_warning_position: None,
            breach_pos: Position::new_i64(0, 0, 0),
        }
    }
}
