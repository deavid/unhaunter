use bevy::prelude::*;
use unspatial_core::position::Position;

use unevents_core::events::roomchanged::InteractionExecutionType;

/// A generic wrapper for targeting entities without knowing their type.
#[derive(Component)]
pub struct Target(pub Entity);

/// A component for entities that can be targeted by position.
#[derive(Component)]
pub struct PositionTarget(pub Position);

/// Something that can be turned on or off.
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct Toggleable {
    pub is_on: bool,
}

/// Event to trigger an interaction with an entity.
#[derive(Debug, Clone, Message)]
pub struct ExecuteInteractionEvent {
    pub entity: Entity,
    pub ietype: InteractionExecutionType,
}
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Triggered;
