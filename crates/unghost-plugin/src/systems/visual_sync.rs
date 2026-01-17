use bevy::prelude::*;
use unghost_core::components::ghost_influence::{GhostInfluence, InfluenceType};
use unghost_core::components::ghost_sprite::GhostSprite;
use unrender_std::components::visuals::{Ethereal, SpectralInfluence, SpectralInfluenceType};

pub(crate) fn ghost_visual_sync(mut q: Query<(&GhostSprite, &mut Ethereal)>) {
    for (gs, mut eth) in q.iter_mut() {
        eth.warp = gs.warp;
        eth.warning_active = gs.hunt_warning_active;
        eth.warning_intensity = gs.hunt_warning_intensity;
        eth.hunt_target = gs.hunt_target;
        eth.calm_time_secs = gs.calm_time_secs;
        eth.hit_delta = gs.repellent_hits_delta;
        eth.miss_delta = gs.repellent_misses_delta;
    }
}

pub(crate) fn ghost_influence_visual_sync(mut q: Query<(&GhostInfluence, &mut SpectralInfluence)>) {
    for (gi, mut si) in q.iter_mut() {
        si.charge_value = gi.charge_value;
        si.influence_type = match gi.influence_type {
            InfluenceType::Attractive => SpectralInfluenceType::Attractive,
            InfluenceType::Repulsive => SpectralInfluenceType::Repulsive,
        };
    }
}
