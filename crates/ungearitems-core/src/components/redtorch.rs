use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Skeleton component for RedTorch - contains only replicated state.
#[derive(Component, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct RedTorch {
    pub enabled: bool,
}

/// Skin component for RedTorch - contains local simulation state, never replicated.
#[derive(Component, Debug, Clone, Reflect, Default)]
pub struct RedTorchSkin {
    pub output_power: f32,
}
