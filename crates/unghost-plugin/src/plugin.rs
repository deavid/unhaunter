use bevy::prelude::*;

use unghost_core::resources::haunt_state::HauntState;
use unghost_core::resources::object_interaction::ObjectInteractionConfig;

use crate::{ghost, ghost_events, ghost_orb, metrics};

pub struct UnhaunterGhostPlugin;

impl Plugin for UnhaunterGhostPlugin {
    fn build(&self, app: &mut App) {
        unghost_core::systems::evidence_decay::app_setup(app);
        ghost::app_setup(app);
        ghost_events::app_setup(app);
        ghost_orb::app_setup(app);
        metrics::register_all(app);
        app.init_resource::<ObjectInteractionConfig>()
            .init_resource::<HauntState>();
    }
}
