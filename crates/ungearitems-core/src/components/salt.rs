use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Data structure for the Salt consumable.
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct SaltData {
    /// Number of salt charges remaining (0-4).
    pub charges: u8,
}

impl Default for SaltData {
    fn default() -> Self {
        Self { charges: 4 }
    }
}

/// Marker component for salt pile entities.
#[derive(Component, Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct SaltPile;

/// Marker inserted once a salt pile is eligible to affect ghosts.
#[derive(Component, Debug, Default)]
pub struct SaltPileArmed;

/// Short grace-period timer that prevents immediate same-frame salt pile consumption.
#[derive(Component, Debug)]
pub struct SaltPileArmingTimer(pub Timer);

/// Marker component for salt particle entities.
#[derive(Component)]
pub struct SaltParticle;

/// Timer component for salt particle lifetime.
#[derive(Component)]
pub struct SaltParticleTimer(pub Timer);

/// Marker component for salt trace entities.
#[derive(Component, Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct SaltyTrace;

/// Component to store the intensity of the green UV glow for SaltyTrace entities.
#[derive(Component)]
pub struct UVReactive(pub f32);

/// Timer component to track the lifetime of a SaltyTrace entity.
#[derive(Component)]
pub struct SaltyTraceTimer(pub Timer);
