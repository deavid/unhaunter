use bevy::prelude::*;
use unaudiospatial_core::emitter::AudioEmitter;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use ungear_core::components::core::{GearSprite, StatusText};
use ungear_core::types::gear::equipment::EquipmentPosition;
use ungear_core::types::gear::sprite_id::GearSpriteID;
use ungearitems_core::components::quartz::{QuartzStoneData, QuartzStoneSkin};
use unghost_core::components::ghost_sprite::GhostSprite;
use unghost_core::tags::GhostTag;
use unmetrics_core::metrics::SendMetric;
use unreplicon_core::ownership::LocallyOwned;
use unreplicon_core::resources::LocalPlayerRole;
use unspatial_core::position::Position;

use crate::metrics;

const MAX_CRACKS: u8 = 4;

pub(crate) trait QuartzStoneDataExt {
    fn aux_quartz_update(
        &mut self,
        skin: &mut QuartzStoneSkin,
        gear_pos: &Position,
        ghost_pos: &Position,
        ghost: &mut GhostSprite,
        dt: f32,
    );
}

impl QuartzStoneDataExt for QuartzStoneData {
    fn aux_quartz_update(
        &mut self,
        skin: &mut QuartzStoneSkin,
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
        if skin.cracked_time > 0.0 {
            skin.cracked_time -= dt;
            let strength = (strength * 1.0).min(ghost.hunting);
            ghost.hunting -= strength;
        } else if ghost.hunt_target {
            let strength = (strength * 8.0).min(ghost.hunting);
            ghost.hunting -= strength;
            skin.energy_absorbed += strength;
        } else {
            let strength = (strength * 0.1).min(ghost.hunting);
            ghost.hunting -= strength;
            skin.energy_absorbed += strength;
        }

        const RESTORE_SPEED: f32 = 0.1;
        skin.energy_absorbed -= (RESTORE_SPEED * dt).min(skin.energy_absorbed);
        // TODO: Spwan here a red particle from the ghost that travels to the quartz ..
        // stone to show the energy of the ghost being drawn.
    }
}

pub(crate) fn update_quartz_skeleton(
    mut gs_audio: AudioEmitter,
    difficulty: Res<CurrentDifficulty>,
    mut q_quartz: Query<
        (
            &mut QuartzStoneData,
            &mut QuartzStoneSkin,
            &Position,
            &EquipmentPosition,
        ),
        With<LocallyOwned>,
    >,
    mut q_ghost: Query<(&Position, &mut GhostSprite), With<GhostTag>>,
) {
    let measure = metrics::UPDATE_QUARTZ_AND_GHOST.time_measure();
    let dt = gs_audio.time.delta_secs();
    for (mut quartz, mut skin, pos, _ep) in q_quartz.iter_mut() {
        // Update logic
        if skin.energy_absorbed > 10.0 * difficulty.0.ghost_hunt_duration().sqrt()
            && quartz.cracks <= MAX_CRACKS
        {
            skin.energy_absorbed = 0.0;
            skin.cracked_time = 5.0;
            // Increment cracks
            quartz.cracks += 1;

            // Play cracking sound
            gs_audio.play_audio("sounds/quartz_crack.ogg".into(), 1.0, pos);
        }

        for (ghost_pos, mut ghost) in q_ghost.iter_mut() {
            quartz.aux_quartz_update(&mut skin, pos, ghost_pos, &mut ghost, dt);
        }
    }

    measure.end_ms();
}

pub(crate) fn update_quartz_skin(
    mut q_quartz: Query<(
        &QuartzStoneData,
        &QuartzStoneSkin,
        &mut StatusText,
        &mut GearSprite,
    )>,
) {
    for (quartz, skin, mut status, mut sprite) in q_quartz.iter_mut() {
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
            energy = skin.energy_absorbed - skin.cracked_time
        );

        sprite.0 = match quartz.cracks {
            0 => GearSpriteID::QuartzStone0.to_visual_key(),
            1 => GearSpriteID::QuartzStone1.to_visual_key(),
            2 => GearSpriteID::QuartzStone2.to_visual_key(),
            3 => GearSpriteID::QuartzStone3.to_visual_key(),
            _ => GearSpriteID::QuartzStone4.to_visual_key(),
        };
    }
}

pub(crate) fn hydrate_quartz_skin(
    mut commands: Commands,
    q_added: Query<Entity, (Added<QuartzStoneData>, Without<QuartzStoneSkin>)>,
) {
    for entity in q_added.iter() {
        commands.entity(entity).insert(QuartzStoneSkin::default());
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, (update_quartz_skeleton, hydrate_quartz_skin));
    app.add_systems(
        Update,
        update_quartz_skin.run_if(resource_exists::<LocalPlayerRole>),
    );
}
