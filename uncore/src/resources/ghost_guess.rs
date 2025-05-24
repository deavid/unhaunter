use bevy::{prelude::*, utils::HashSet};

use crate::types::{evidence::Evidence, ghost::types::GhostType};

#[derive(Debug, Resource, Default)]
pub struct GhostGuess {
    pub ghost_type: Option<GhostType>,
    pub evidences_found: HashSet<Evidence>,
    pub evidences_missing: HashSet<Evidence>,
}
