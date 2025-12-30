use bevy::prelude::*;

/// Data structure for the Sage Bundle consumable.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct SageBundleData {
    /// Indicates whether the Sage Bundle is currently active (burning).
    pub is_active: bool,
    /// Timer for the burn duration.
    pub burn_timer: Timer,
    /// If the sage has been completely burned.
    pub consumed: bool,
    /// Amount of particles of smoke produced. Used to pace the smoke production.
    pub smoke_produced: usize,
}

impl Default for SageBundleData {
    fn default() -> Self {
        Self {
            is_active: false,
            burn_timer: Timer::from_seconds(8.0, TimerMode::Once),
            consumed: false,
            smoke_produced: 0,
        }
    }
}

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct SageSmokeParticle;

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct SmokeParticleTimer(pub Timer);
