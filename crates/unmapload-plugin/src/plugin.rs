use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unmapload_core::assets::{MapAssets, MissionAssets};
use unmapload_core::events::loadlevel::{LevelLoadedEvent, LoadLevelEvent, MapEntitiesReadyEvent};
use unmission_core::events::{LevelReadyEvent, MapGeometryInitializedEvent};
use unorchestrator_core::UIContextState;
use untmxmap_core::events::LevelDataEvent;

/// Core plugin for map loading functionality
///
/// This plugin registers all the necessary systems and resources for map loading,
/// including level setup, entity spawning, and post-processing.
pub struct UnhaunterMapLoadCorePlugin;

impl Plugin for UnhaunterMapLoadCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LoadLevelEvent>()
            .add_message::<LevelDataEvent>()
            .add_message::<LevelLoadedEvent>()
            .add_message::<LevelReadyEvent>()
            .add_message::<MapGeometryInitializedEvent>()
            .add_message::<MapEntitiesReadyEvent>();

        // Call the main app_setup from the module
        crate::module::app_setup(app);
    }
}

/// Plugin for map loading assets
pub struct UnhaunterMapLoadPlugin;

impl Plugin for UnhaunterMapLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(UIContextState::EngineBoot)
                .continue_to_state(UIContextState::MainMenu)
                .load_collection::<MapAssets>()
                .load_collection::<MissionAssets>(),
        );

        // Note: MissionAssets and MapAssets are loaded at EngineBoot above.
        // No bevy_asset_loader loading state is used for MissionLoading — the
        // transition MissionLoading → InGame is driven exclusively by the map
        // hydration pipeline (after_level_ready) to prevent a race where the
        // asset loader would complete instantly and transition to InGame before
        // board resources are allocated, causing a self-transition that wipes them.

        // Register rendering systems (only for non-headless clients)
        crate::level_finalization::app_setup_render_only(app);
    }
}
