use bevy::prelude::*;

/// Local-only presentation timer for ghost fade-out.
///
/// This component is created by presentation when it observes the logic-owned
/// `GhostDeathSignal`. It is never owned or ticked by logic.
#[derive(Component)]
pub struct GhostDying {
    pub timer: Timer,
}

impl GhostDying {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
