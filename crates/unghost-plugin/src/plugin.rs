use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use untypes_core::states::AppState;

use unghost_core::resources::haunt_state::HauntState;
use unghost_core::resources::object_interaction::ObjectInteractionConfig;

use crate::{ghost_events, ghost_orb, metrics};
use unghost_core::assets::GhostAssets;

pub struct UnhaunterGhostPlugin;

impl Plugin for UnhaunterGhostPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::Loading).load_collection::<GhostAssets>(),
        );
        crate::systems::hydration::app_setup(app);
        unghost_core::systems::evidence_decay::app_setup(app);
        crate::systems::ghost_ai::app_setup(app);
        ghost_events::app_setup(app);
        ghost_orb::app_setup(app);
        metrics::register_all(app);
        app.init_resource::<ObjectInteractionConfig>()
            .init_resource::<HauntState>();
    }
}
