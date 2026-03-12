use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use unspatial_core::position::Position;

use unevents_core::events::roomchanged::InteractionExecutionType;

/// A generic wrapper for targeting entities without knowing their type.
#[derive(Component)]
pub struct Target(pub Entity);

/// A component for entities that can be targeted by position.
#[derive(Component)]
pub struct PositionTarget(pub Position);

/// Something that can be turned on or off.
#[derive(Component, Debug, Clone, Copy, Reflect, Default, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Toggleable {
    pub is_on: bool,
}

/// Event to trigger an interaction with an entity.
#[derive(Debug, Clone, Message)]
pub struct ExecuteInteractionEvent {
    pub entity: Entity,
    pub ietype: InteractionExecutionType,
    /// If Some, force the interaction to transition to this specific tile UID.
    /// Useful for network synchronization.
    pub force_tuid: Option<u32>,
}
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Triggered;

/// Authority level for executing interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Authority {
    /// Host: Can mutate state, play sounds, and transitions.
    Host,
    /// Client: Read-only, visual updates only.
    Client,
}
