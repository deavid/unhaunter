use bevy::prelude::*;
use unfoundation_core::types::gear::{EquipmentPosition, GearSpriteID};
use ungear_core::gear_stuff::GearStuff;
use ungear_core::{GearSprite, StatusText};
pub use ungearitems_core::components::quartz::QuartzStoneData;
use unghost_core::components::GhostSprite;
use unspatial_core::Position;
use untags_core::GhostTag;

const MAX_CRACKS: u8 = 4;

pub trait QuartzStoneDataExt {
    fn aux_quartz_update(
        &mut self,
        gear_pos: &Position,
        ghost_pos: &Position,
        ghost: &mut GhostSprite,
        dt: f32,
    );
}

impl QuartzStoneDataExt for QuartzStoneData {
    fn aux_quartz_update(
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
