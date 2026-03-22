use bevy::prelude::*;
use unplayer_core::components::PlayerSprite;
use unrender_std::components::visuals::Viewer;

pub(crate) fn sync_vitals_to_viewer(mut q: Query<(&PlayerSprite, &mut Viewer)>) {
    for (ps, mut viewer) in q.iter_mut() {
        viewer.id = ps.network_id;
        viewer.health = ps.health;
        viewer.sanity = ps.sanity;
    }
}
