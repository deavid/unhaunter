use bevy::prelude::*;

pub mod components;
pub mod gear_details;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct GearStateExportSet;
