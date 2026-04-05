use bevy::prelude::*;

/// Component for door lock visual indicator
#[derive(Component, Debug)]
pub(crate) struct LockIndicator {
    pub pulse_timer: Timer,
    pub base_alpha: f32,
}

impl Default for LockIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl LockIndicator {
    pub(crate) fn new() -> Self {
        Self {
            pulse_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            base_alpha: 0.8,
        }
    }
}
