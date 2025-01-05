use bevy::prelude::*;

use crate::types::root::map::{Map, Sheet};

#[derive(Resource, Clone, Debug, Default)]
pub struct Maps {
    pub maps: Vec<Map>,
    pub sheets: Vec<Sheet>,
}
