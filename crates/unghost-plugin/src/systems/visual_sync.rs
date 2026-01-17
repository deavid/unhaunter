use bevy::prelude::*;
use unghost_core::components::ghost_sprite::GhostSprite;
use unrender_std::components::visuals::Ethereal;

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
