use crate::types::root::map::{Map, Sheet};
use bevy::prelude::*;

#[derive(Resource, Clone, Debug, Default)]
pub struct Maps {
    pub maps: Vec<Map>,
    pub sheets: Vec<Sheet>,
}
