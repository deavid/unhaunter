//! Generic components for entity interactions.

pub mod interactivestuff;

use bevy::prelude::*;
use unspatial_core::Position;

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

/// Marker for a trigger event on an item.
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct Triggered;
