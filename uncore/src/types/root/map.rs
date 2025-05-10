use crate::assets::{tmxmap::TmxMap, tsxsheet::TsxSheet};
use crate::types::mission_data::MissionData;
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct Map {
    pub name: String,
    pub path: String,
    pub handle: Handle<TmxMap>,
    /// Mission data for this map.
    pub mission_data: MissionData,
}

#[derive(Clone, Debug)]
pub struct Sheet {
    pub path: String,
    pub handle: Handle<TsxSheet>,
}
