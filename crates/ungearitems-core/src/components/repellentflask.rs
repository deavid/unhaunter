use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use unghost_core::types::ghost::types::GhostType;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct RepellentFlask {
    /// The ghost type that the repellent is effective against. It is never set to None even if emptied.
    pub liquid_content: Option<GhostType>,
    pub active: bool,
    pub qty: i32,
}

impl RepellentFlask {
    pub const MAX_QTY: i32 = 400;
}
