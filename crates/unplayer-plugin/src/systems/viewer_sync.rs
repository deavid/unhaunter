use bevy::prelude::*;
use unplayer_core::components::PlayerSprite;
use unplayer_core::components::PlayerVitals;
use unrender_std::components::visuals::Viewer;

pub(crate) fn sync_vitals_to_viewer(mut q: Query<(&PlayerSprite, &PlayerVitals, &mut Viewer)>) {
    for (ps, vitals, mut viewer) in q.iter_mut() {
        viewer.id = ps.network_id;
        viewer.health = vitals.health;
        viewer.sanity = vitals.sanity;
    }
}
