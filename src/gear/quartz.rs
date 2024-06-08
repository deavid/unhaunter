//! This module defines the `QuartzStoneData` struct and its associated logic,
//! representing the Quartz Stone consumable item in the game.

use bevy::prelude::*;

use super::{Gear, GearKind, GearSpriteID, GearStuff, GearUsable};
use crate::board::Position;

/// Data structure for the Quartz Stone consumable.
#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct QuartzStoneData {
    /// Number of cracks in the stone (0-3).
    pub cracks: u8,
}

impl GearUsable for QuartzStoneData {
    fn get_display_name(&self) -> &'static str {
        "Quartz Stone"
    }

    fn get_description(&self) -> &'static str {
        "A protective charm that absorbs the ghost's hunting energy, preventing or shortening hunts. The stone gradually cracks and eventually breaks after repeated uses."
    }

    fn get_status(&self) -> String {
        if self.cracks >= 3 {
            return "Shattered".to_string();
        }
        format!("Cracks: {}", self.cracks)
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {
        // Quartz Stone is always active, no trigger action needed.
    }

    fn update(
        &mut self,
        gs: &mut GearStuff,
        pos: &Position,
        _ep: &super::playergear::EquipmentPosition,
    ) {
        // Check proximity to ghost
        let distance_to_ghost = gs.bf.breach_pos.distance(pos);
        if distance_to_ghost <= 7.0 && self.cracks < 3 {
            // Increment cracks
            self.cracks += 1;

            // Play cracking sound
            gs.play_audio("sounds/quartz_crack.ogg".into(), 1.0, pos);
        }
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.cracks {
            0 => GearSpriteID::QuartzStone0,
            1 => GearSpriteID::QuartzStone1,
            2 => GearSpriteID::QuartzStone2,
            3 => GearSpriteID::QuartzStone3,
            _ => GearSpriteID::QuartzStone4, // Shattered
        }
    }

    fn _box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<QuartzStoneData> for Gear {
    fn from(value: QuartzStoneData) -> Self {
        Gear::new_from_kind(GearKind::QuartzStone(value))
    }
}
