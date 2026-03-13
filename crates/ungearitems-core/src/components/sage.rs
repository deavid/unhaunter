use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Data structure for the Sage Bundle consumable.
#[derive(Component, Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct SageBundleData {
    /// Indicates whether the Sage Bundle is currently active (burning).
    pub is_active: bool,
    /// If the sage has been completely burned.
    pub consumed: bool,
}

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct SageBundleSkin {
    /// Timer for the burn duration.
    pub burn_timer: Timer,
    /// Amount of particles of smoke produced. Used to pace the smoke production.
    pub smoke_produced: usize,
}

impl SageBundleSkin {
    pub fn new() -> Self {
        Self {
            burn_timer: Timer::from_seconds(8.0, TimerMode::Once),
            smoke_produced: 0,
        }
    }
}

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct SageSmokeParticle;

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct SmokeParticleTimer(pub Timer);
