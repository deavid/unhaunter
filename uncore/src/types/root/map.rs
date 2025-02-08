use crate::assets::{tmxmap::TmxMap, tsxsheet::TsxSheet};
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct Map {
    pub name: String,
    pub path: String,
    pub handle: Handle<TmxMap>,
}

#[derive(Clone, Debug)]
pub struct Sheet {
    pub path: String,
    pub handle: Handle<TsxSheet>,
}
