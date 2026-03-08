use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for the ghost's visual breach effect.
#[derive(Component, Debug, Default, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct GhostBreach;
