use bevy::prelude::*;

/// Marker component for a player entity that is currently in the truck/van.
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct InTruck;
