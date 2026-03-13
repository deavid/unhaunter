use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Skeleton component for UVTorch - contains only replicated state.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct UVTorch {
    pub enabled: bool,
}

/// Skin component for UVTorch - contains local simulation state, never replicated.
#[derive(Component, Debug, Clone, Reflect, Default)]
pub struct UVTorchSkin {
    pub output_power: f32,
}
