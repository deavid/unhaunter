use bevy::prelude::*;
use uncore::difficulty::DifficultyStruct;
use ungear::types::gear::Gear;
use ungearitems::from_gearkind::FromGearKind;

#[derive(Debug, Resource, Clone)]
pub struct TruckGear {
    pub inventory: Vec<Gear>,
}

impl TruckGear {
    pub fn from_difficulty(difficulty: &DifficultyStruct) -> Self {
        Self {
            inventory: difficulty
                .truck_gear
                .iter()
                .map(|gk| Gear::from_gearkind(*gk))
                .collect::<Vec<_>>(),
        }
    }
}
