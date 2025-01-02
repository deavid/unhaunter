use crate::{difficulty::DifficultyStruct, gear::Gear};
use bevy::prelude::*;

#[derive(Debug, Resource, Clone)]
pub struct TruckGear {
    pub inventory: Vec<Gear>,
}

impl TruckGear {
    pub fn from_difficulty(difficulty: &DifficultyStruct) -> Self {
        Self {
            inventory: difficulty.truck_gear.clone(),
        }
    }
}
