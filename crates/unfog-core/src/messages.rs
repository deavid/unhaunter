use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Message emitted when a miasma hazard particle damages a player.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct MiasmaTakeDamageMessage {
    /// The entity that took damage.
    pub target_entity: Entity,
    /// The amount of damage taken per second.
    pub damage: f32,
}

/// Request from client to server to spawn a miasma hazard particle.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct RequestSpawnHazardParticle {
    /// The position to spawn the particle at.
    pub position: Vec3,
}
