//! This module defines the `QuartzStoneData` struct and its associated logic,
//! representing the Quartz Stone consumable item in the game.

use super::{EquipmentPosition, Gear, GearKind, GearSpriteID, GearUsable};
use bevy::prelude::*;
use uncore_components::{GearSprite, StatusText};
use ungear::gear_stuff::GearStuff;
use unghost_core::components::GhostSprite;
use unspatial::Position;
use untags::GhostTag;

const MAX_CRACKS: u8 = 4;

/// Data structure for the Quartz Stone consumable.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct QuartzStoneData {
    /// Number of cracks in the stone (0-3).
    pub cracks: u8,
    /// Bonus time for recently cracked
    pub cracked_time: f32,
    /// Amount of energy absorbed from the ghost - what produces the cracks.
    pub energy_absorbed: f32,
}

impl QuartzStoneData {
    pub fn aux_quartz_update(
        &mut self,
        gear_pos: &Position,
        ghost_pos: &Position,
        ghost: &mut GhostSprite,
        dt: f32,
    ) {
        const MIN_DIST: f32 = 5.0;
        const MIN_DIST2: f32 = MIN_DIST * MIN_DIST;
        let distance2 = gear_pos.distance2(ghost_pos);
        let dist_adj = (distance2 + MIN_DIST2) / MIN_DIST2;
        let dist_adj_recip = dist_adj.recip() - 0.2;
        let stone_health = (MAX_CRACKS - self.cracks) as f32 / MAX_CRACKS as f32;
        let strength = ghost.hunting
            * dt
            * dist_adj_recip.clamp(0.0, 1.0)
            * stone_health.clamp(0.0, 1.0).sqrt();
        if self.cracked_time > 0.0 {
            self.cracked_time -= dt;
            let strength = (strength * 1.0).min(ghost.hunting);
            ghost.hunting -= strength;
        } else if ghost.hunt_target {
            let strength = (strength * 8.0).min(ghost.hunting);
            ghost.hunting -= strength;
            self.energy_absorbed += strength;
        } else {
            let strength = (strength * 0.1).min(ghost.hunting);
            ghost.hunting -= strength;
            self.energy_absorbed += strength;
        }

        const RESTORE_SPEED: f32 = 0.1;
        self.energy_absorbed -= (RESTORE_SPEED * dt).min(self.energy_absorbed);
        // TODO: Spwan here a red particle from the ghost that travels to the quartz ..
        // stone to show the energy of the ghost being drawn.
    }
}

pub fn update_quartz(
    mut gs: GearStuff,
    mut q_quartz: Query<(
        &mut QuartzStoneData,
        &mut StatusText,
        &mut GearSprite,
        &Position,
        &EquipmentPosition,
    )>,
    mut q_ghost: Query<(&Position, &mut GhostSprite), With<GhostTag>>,
) {
    let dt = gs.time.delta_secs();
    for (mut quartz, mut status, mut sprite, pos, _ep) in q_quartz.iter_mut() {
        // Update logic
        if quartz.energy_absorbed > 10.0 * gs.difficulty.0.ghost_hunt_duration.sqrt()
            && quartz.cracks <= MAX_CRACKS
        {
            quartz.energy_absorbed = 0.0;
            quartz.cracked_time = 5.0;
            // Increment cracks
            quartz.cracks += 1;

            // Play cracking sound
            gs.play_audio("sounds/quartz_crack.ogg".into(), 1.0, pos);
        }

        for (ghost_pos, mut ghost) in q_ghost.iter_mut() {
            quartz.aux_quartz_update(pos, ghost_pos, &mut ghost, dt);
        }

        // Update StatusText
        let state = match quartz.cracks {
            0 => "Pure",
            1 => "Used once",
            2 => "Used twice",
            3 => "Cracked, one use remaining",
            4 => "Shattered - Unusable",
            _ => "unknown",
        };
        status.0 = format!(
            "State: {state}\nEnergy absorbed: {energy:.1}",
            energy = quartz.energy_absorbed - quartz.cracked_time
        );

        // Update GearSprite
        sprite.0 = match quartz.cracks {
            0 => GearSpriteID::QuartzStone0,
            1 => GearSpriteID::QuartzStone1,
            2 => GearSpriteID::QuartzStone2,
            3 => GearSpriteID::QuartzStone3,
            // Shattered
            _ => GearSpriteID::QuartzStone4,
        };
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_quartz);
}

impl GearUsable for QuartzStoneData {
    fn get_display_name(&self) -> &'static str {
        "Quartz Stone"
    }

    fn get_description(&self) -> &'static str {
        "A protective charm that absorbs the ghost's hunting energy, preventing or shortening hunts. The stone gradually cracks and eventually breaks after repeated uses."
    }

    fn get_status(&self) -> String {
        match self.cracks {
            0 => "Pure".to_string(),
            1 => "Used once".to_string(),
            2 => "Used twice".to_string(),
            3 => "Used thrice".to_string(),
            _ => "Shattered".to_string(),
        }
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {}

    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.cracks {
            0 => GearSpriteID::QuartzStone0,
            1 => GearSpriteID::QuartzStone1,
            2 => GearSpriteID::QuartzStone2,
            3 => GearSpriteID::QuartzStone3,
            _ => GearSpriteID::QuartzStone4,
        }
    }

    fn update(&mut self, _gs: &mut GearStuff, _pos: &Position, _ep: &EquipmentPosition) {}

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<QuartzStoneData> for Gear {
    fn from(value: QuartzStoneData) -> Self {
        Gear::new_from_kind(GearKind::QuartzStone, value.box_clone())
    }
}
