use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for a player entity that is currently in the truck/van.
#[derive(Component, Debug, Clone, Copy, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct InTruck;
