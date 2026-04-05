use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Logic-owned death state for ghost entities and their breach visuals.
///
/// This is the authoritative signal that an entity has entered its dying phase.
/// Logic owns insertion and despawn timing; presentation may observe it and create
/// local-only fade timers or effects.
#[derive(Component, Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct GhostDeathSignal {
    /// `Time::elapsed_secs_f64()` when the dying phase started.
    pub started_at_secs: f64,
    /// Duration of the dying phase in seconds before logic despawns the entity.
    pub duration_secs: f32,
}

impl GhostDeathSignal {
    pub fn new(current_secs: f64, duration_secs: f32) -> Self {
        Self {
            started_at_secs: current_secs,
            duration_secs,
        }
    }

    pub fn is_finished(&self, current_secs: f64) -> bool {
        current_secs >= self.started_at_secs + self.duration_secs as f64
    }
}
