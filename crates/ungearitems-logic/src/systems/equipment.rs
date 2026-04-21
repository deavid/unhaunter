use bevy::prelude::*;
use enum_iterator::Sequence;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use ungear_core::components::core::{Battery, Electronic};
use ungear_core::types::gear::equipment::EquipmentPosition;
use ungearitems_core::components::flashlight::{Flashlight, FlashlightStatus};
use ungearitems_core::components::quartz::{QuartzStoneData, QuartzStoneSkin};
use ungearitems_core::components::redtorch::RedTorch;
use ungearitems_core::components::uvtorch::UVTorch;
use ungearitems_core::events::QuartzCrackedEvent;
use unghost_core::components::logic::ghost_sprite::GhostSprite;
use unghost_core::tags::GhostTag;
use uninteraction_core::interaction::{Toggleable, Triggered};
use unmetrics_core::metrics::SendMetric;
use unreplicon_core::ownership::LocallyOwned;
use unspatial_core::position::Position;

use crate::metrics;

const MAX_CRACKS: u8 = 4;

pub(crate) fn update_uvtorch_skeleton(
    mut q_uvtorch: Query<(&mut UVTorch, &mut Battery, &Toggleable), With<LocallyOwned>>,
) {
    let measure = metrics::UVTORCH_UPDATE.time_measure();
    for (mut uvtorch, mut battery, toggle) in q_uvtorch.iter_mut() {
        uvtorch.enabled = toggle.is_on;
        battery.drain_rate = if uvtorch.enabled { 0.0001 } else { 0.0 };
    }
    measure.end_ms();
}

pub(crate) fn update_redtorch_skeleton(
    mut q_redtorch: Query<(&mut RedTorch, &mut Battery, &Toggleable), With<LocallyOwned>>,
) {
    let measure = metrics::REDTORCH_UPDATE.time_measure();
    for (mut redtorch, mut battery, toggle) in q_redtorch.iter_mut() {
        redtorch.enabled = toggle.is_on;
        battery.drain_rate = if redtorch.enabled { 0.0001 } else { 0.0 };
    }
    measure.end_ms();
}

pub(crate) fn update_flashlight_skeleton(
    mut commands: Commands,
    mut q_flashlight: Query<
        (
            Entity,
            &mut Flashlight,
            &mut Toggleable,
            &Battery,
            &Electronic,
            Option<&Triggered>,
        ),
        With<LocallyOwned>,
    >,
) {
    let measure = metrics::FLASHLIGHT_UPDATE.time_measure();
    for (entity, mut flashlight, mut toggle, battery, electronic, triggered) in
        q_flashlight.iter_mut()
    {
        if triggered.is_some() && electronic.glitch_timer <= 0.0 {
            let next_status = flashlight.status.next().unwrap_or_default();
            let is_battery_ok = battery.level > 0.0;
            if next_status == FlashlightStatus::Off || is_battery_ok {
                flashlight.status = next_status;
            } else if flashlight.status != FlashlightStatus::Off {
                flashlight.status = FlashlightStatus::Off;
            }
            commands.entity(entity).remove::<Triggered>();
        }
        toggle.is_on = flashlight.status != FlashlightStatus::Off;
    }
    measure.end_ms();
}

pub(crate) fn update_videocam_skeleton(
    mut q_videocam: Query<(&Toggleable, &mut Battery), With<LocallyOwned>>,
) {
    let measure = metrics::VIDEOCAM_UPDATE.time_measure();
    for (toggle, mut battery) in q_videocam.iter_mut() {
        battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };
    }
    measure.end_ms();
}

pub(crate) fn update_quartz_skeleton(
    time: Res<Time>,
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
    mut ev_cracked: MessageWriter<QuartzCrackedEvent>,
) {
    let measure = metrics::UPDATE_QUARTZ_AND_GHOST.time_measure();
    let dt = time.delta_secs();
    for (mut quartz, mut skin, pos, _ep) in q_quartz.iter_mut() {
        if skin.energy_absorbed > 10.0 * difficulty.0.ghost_hunt_duration().sqrt()
            && quartz.cracks <= MAX_CRACKS
        {
            skin.energy_absorbed = 0.0;
            skin.cracked_time = 5.0;
            quartz.cracks += 1;
            ev_cracked.write(QuartzCrackedEvent {
                position: [pos.x, pos.y, pos.z, pos.visual_priority],
            });
        }

        for (ghost_pos, mut ghost) in q_ghost.iter_mut() {
            aux_quartz_update(&mut quartz, &mut skin, pos, ghost_pos, &mut ghost, dt);
        }
    }
    measure.end_ms();
}

fn aux_quartz_update(
    quartz: &mut QuartzStoneData,
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
    let stone_health = (MAX_CRACKS - quartz.cracks) as f32 / MAX_CRACKS as f32;
    let strength =
        ghost.hunting * dt * dist_adj_recip.clamp(0.0, 1.0) * stone_health.clamp(0.0, 1.0).sqrt();
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
}
