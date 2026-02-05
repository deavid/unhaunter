use bevy::prelude::*;
use unghost_core::components::ghost_influence::{GhostInfluence, InfluenceType};
use unghost_core::components::ghost_sprite::GhostSprite;
use unmetrics_core::metrics::SendMetric;
use unrender_std::components::visuals::{Ethereal, SpectralInfluence, SpectralInfluenceType};

use crate::metrics;

pub(crate) fn ghost_visual_sync(mut q: Query<(&GhostSprite, &mut Ethereal)>) {
    let measure = metrics::GHOST_VISUAL_SYNC.time_measure();
    for (gs, mut eth) in q.iter_mut() {
        eth.warp = gs.warp;
        eth.warning_active = gs.hunt_warning_active;
        eth.warning_intensity = gs.hunt_warning_intensity;
        eth.hunt_target = gs.hunt_target;
        eth.calm_time_secs = gs.calm_time_secs;
        eth.hit_delta = gs.repellent_hits_delta;
        eth.miss_delta = gs.repellent_misses_delta;
    }
    measure.end_ms();
}

pub(crate) fn ghost_influence_visual_sync(mut q: Query<(&GhostInfluence, &mut SpectralInfluence)>) {
    let measure = metrics::GHOST_INFLUENCE_VISUAL_SYNC.time_measure();
    for (gi, mut si) in q.iter_mut() {
        si.charge_value = gi.charge_value;
        si.influence_type = match gi.influence_type {
            InfluenceType::Attractive => SpectralInfluenceType::Attractive,
            InfluenceType::Repulsive => SpectralInfluenceType::Repulsive,
        };
    }
    measure.end_ms();
}
