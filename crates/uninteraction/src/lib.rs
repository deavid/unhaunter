//! Generic components for entity interactions.

use bevy::prelude::*;
use unspatial::Position;

/// A generic wrapper for targeting entities without knowing their type.
#[derive(Component)]
pub struct Target(pub Entity);

/// A component for entities that can be targeted by position.
#[derive(Component)]
pub struct PositionTarget(pub Position);