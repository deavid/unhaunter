use bevy::prelude::*;

use crate::{ghost, ghost_events, ghost_orb, metrics};

pub struct UnhaunterGhostPlugin;

impl Plugin for UnhaunterGhostPlugin {
    fn build(&self, app: &mut App) {
        ghost::app_setup(app);
        ghost_events::app_setup(app);
        ghost_orb::app_setup(app);
        metrics::register_all(app);
    }
}
