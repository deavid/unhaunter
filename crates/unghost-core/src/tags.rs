use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component, Default)]
pub struct GhostTag;
