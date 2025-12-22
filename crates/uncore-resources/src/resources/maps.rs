use bevy::prelude::*;
use uncore_assets::types::root::map::{Map, Sheet};

#[derive(Resource, Clone, Debug, Default)]
pub struct Maps {
    pub maps: Vec<Map>,
    pub sheets: Vec<Sheet>,
}
