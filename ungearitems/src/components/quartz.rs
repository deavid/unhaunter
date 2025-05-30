//! This module defines the `QuartzStoneData` struct and its associated logic,
//! representing the Quartz Stone consumable item in the game.

use super::{Gear, GearKind, GearSpriteID, GearStuff, GearUsable};
use crate::metrics;
use bevy::prelude::*;
use uncore::{
    components::{board::position::Position, ghost_sprite::GhostSprite},
    metric_recorder::SendMetric,
    types::gear::equipmentposition::EquipmentPosition,
};
use ungear::components::{deployedgear::DeployedGearData, playergear::PlayerGear};

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

impl GearUsable for QuartzStoneData {
    fn get_display_name(&self) -> &'static str {
        "Quartz Stone"
    }

    fn get_description(&self) -> &'static str {
        "A protective charm that absorbs the ghost's hunting energy, preventing or shortening hunts. The stone gradually cracks and eventually breaks after repeated uses."
    }

    fn get_status(&self) -> String {
        let state = match self.cracks {
            0 => "Pure",
            1 => "Used once",
            2 => "Used twice",
            3 => "Cracked, one use remaining",
            4 => "Shattered - Unusable",
            _ => "unknown",
        };
        format!(
            "State: {state}\nEnergy absorbed: {energy:.1}",
            energy = self.energy_absorbed - self.cracked_time
        )
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {
        // Quartz Stone is always active, no trigger action needed.
    }

    fn update(&mut self, gs: &mut GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        if self.energy_absorbed > 10.0 * gs.difficulty.0.ghost_hunt_duration.sqrt()
            && self.cracks <= MAX_CRACKS
        {
            self.energy_absorbed = 0.0;
            self.cracked_time = 5.0;
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
            // Shattered
            _ => GearSpriteID::QuartzStone4,
        }
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }

    fn aux_quartz_update(
        &mut self,
        gear_pos: &Position,
        ghost_pos: &Position,
        ghost_sprite: &mut GhostSprite,
        dt: f32,
    ) {
        const MIN_DIST: f32 = 5.0;
        const MIN_DIST2: f32 = MIN_DIST * MIN_DIST;
        let distance2 = gear_pos.distance2(ghost_pos);
        let dist_adj = (distance2 + MIN_DIST2) / MIN_DIST2;
        let dist_adj_recip = dist_adj.recip() - 0.2;
        let stone_health = (MAX_CRACKS - self.cracks) as f32 / MAX_CRACKS as f32;
        let strength = ghost_sprite.hunting
            * dt
            * dist_adj_recip.clamp(0.0, 1.0)
            * stone_health.clamp(0.0, 1.0).sqrt();
        if self.cracked_time > 0.0 {
            self.cracked_time -= dt;
            let strength = (strength * 1.0).min(ghost_sprite.hunting);
            ghost_sprite.hunting -= strength;
        } else if ghost_sprite.hunt_target {
            let strength = (strength * 8.0).min(ghost_sprite.hunting);
            ghost_sprite.hunting -= strength;
            self.energy_absorbed += strength;
        } else {
            let strength = (strength * 0.1).min(ghost_sprite.hunting);
            ghost_sprite.hunting -= strength;
            self.energy_absorbed += strength;
        }

        const RESTORE_SPEED: f32 = 0.1;
        self.energy_absorbed -= (RESTORE_SPEED * dt).min(self.energy_absorbed);
        // TODO: Spwan here a red particle from the ghost that travels to the quartz ..
        // stone to show the energy of the ghost being drawn.
    }
}

impl From<QuartzStoneData> for Gear {
    fn from(value: QuartzStoneData) -> Self {
        Gear::new_from_kind(GearKind::QuartzStone, value.box_clone())
    }
}

fn update_quartz_and_ghost(
    mut q_gear1: Query<(&Position, &mut PlayerGear)>,
    mut q_gear2: Query<(&Position, &mut DeployedGearData)>,
    mut q_ghost: Query<(&Position, &mut GhostSprite)>,
    time: Res<Time>,
) {
    let measure = metrics::UPDATE_QUARTZ_AND_GHOST.time_measure();
    let dt = time.delta_secs();
    for (gear_pos, mut playergear) in q_gear1.iter_mut() {
        for (gear, _) in playergear.as_vec_mut().into_iter() {
            if let GearKind::QuartzStone = gear.kind {
                for (ghost_pos, mut ghost_sprite) in q_ghost.iter_mut() {
                    gear.data.as_mut().unwrap().aux_quartz_update(
                        gear_pos,
                        ghost_pos,
                        &mut ghost_sprite,
                        dt,
                    );
                }
            }
        }
    }
    for (gear_pos, mut gear_data) in q_gear2.iter_mut() {
        if let GearKind::QuartzStone = gear_data.gear.kind {
            for (ghost_pos, mut ghost_sprite) in q_ghost.iter_mut() {
                gear_data.gear.data.as_mut().unwrap().aux_quartz_update(
                    gear_pos,
                    ghost_pos,
                    &mut ghost_sprite,
                    dt,
                );
            }
        }
    }
    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_quartz_and_ghost);
}
