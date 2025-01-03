use bevy::prelude::*;

use crate::types::root::map::Map;

#[derive(Resource, Clone, Debug, Default)]
pub struct Maps {
    pub maps: Vec<Map>,
}
