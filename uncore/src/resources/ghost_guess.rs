use bevy::prelude::*;

use crate::types::ghost::types::GhostType;

#[derive(Debug, Resource, Default)]
pub struct GhostGuess {
    pub ghost_type: Option<GhostType>,
}
