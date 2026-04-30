use bevy::prelude::*;
use bevy_platform::collections::HashSet;
use serde::{Deserialize, Serialize};

use crate::{evidence::Evidence, ghost::GhostType};

#[derive(Debug, Component, Default, Clone, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect(Component, Default, PartialEq)]
pub struct GhostGuess {
    pub ghost_type: Option<GhostType>,
    pub evidences_found: HashSet<Evidence>,
    pub evidences_missing: HashSet<Evidence>,
    pub ghosts_discarded: HashSet<GhostType>,
}
