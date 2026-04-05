use bevy::prelude::*;
use unghost_core::components::logic::ghost_death::GhostDeathSignal;
use unghost_core::components::logic::ghost_sprite::GhostBehaviorDynamics;
use unghost_core::components::logic::ghost_sprite::GhostSprite;
use unghost_core::components::presentation::ghost_dying::GhostDying;
use unghost_core::components::presentation::spectral::SpectralClarity;
use unghost_core::tags::GhostTag;
use unmetrics_core::metrics::SendMetric;
use unrender_std::components::visuals::{Emissive, Ethereal};

use crate::metrics;

pub(crate) fn ghost_visual_sync(
    mut q: Query<(&GhostSprite, &mut Ethereal, Option<&mut Emissive>)>,
) {
    let measure = metrics::GHOST_VISUAL_SYNC.time_measure();
    for (gs, mut eth, mut o_emissive) in q.iter_mut() {
        eth.warp = gs.warp;
        eth.warning_active = gs.hunt_warning_active;
        eth.warning_intensity = gs.hunt_warning_intensity;
        eth.threat_active = gs.hunt_target;
        eth.threat_calm_mix = (gs.calm_time_secs / 10.0).clamp(0.0, 1.0);
        eth.hit_delta = gs.repellent_hits_delta;
        eth.miss_delta = gs.repellent_misses_delta;

        if let Some(emissive) = o_emissive.as_mut() {
            if gs.hunt_warning_active || gs.hunt_target {
                emissive.color = Color::srgb(1.0, 0.0, 0.0);
                emissive.intensity = 1.0;
                emissive.pulse_speed = 4.0;
            } else {
                emissive.color = Color::WHITE;
                emissive.intensity = 0.0;
                emissive.pulse_speed = 0.0;
            }
        }
    }
    measure.end_ms();
}

pub(crate) fn ghost_clarity_sync(
    mut q_ghost: Query<
        (
            &mut SpectralClarity,
            &GhostBehaviorDynamics,
            Option<&GhostDying>,
            Option<&GhostDeathSignal>,
        ),
        With<GhostTag>,
    >,
) {
    for (mut clarity, dynamics, dying_visual, dying_logic) in q_ghost.iter_mut() {
        clarity.uv = dynamics.uv_ectoplasm_clarity;
        clarity.rl = dynamics.rl_presence_clarity;
        clarity.alpha = if let Some(dying) = dying_visual {
            dying.timer.remaining_secs() / dying.timer.duration().as_secs_f32()
        } else if dying_logic.is_some() {
            0.0
        } else {
            dynamics.visual_alpha_multiplier
        };
    }
}
