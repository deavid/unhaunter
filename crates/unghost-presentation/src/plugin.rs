use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unghost_core::assets::GhostAssets;
use unorchestrator_core::UIContextState;

use crate::metrics;

pub struct GhostPresentationPlugin;

impl Plugin for GhostPresentationPlugin {
    fn build(&self, app: &mut App) {
        metrics::register_presentation(app);
        app.add_loading_state(
            LoadingState::new(UIContextState::EngineBoot).load_collection::<GhostAssets>(),
        );
        crate::systems::app_setup(app);
        app.add_systems(
            Update,
            (
                crate::systems::visual_sync::ghost_visual_sync,
                crate::systems::visual_sync::ghost_clarity_sync,
            )
                .run_if(in_state(UIContextState::InGame)),
        );
        app.init_resource::<crate::systems::ghost_orb::OrbSpawnTimer>();
    }
}
