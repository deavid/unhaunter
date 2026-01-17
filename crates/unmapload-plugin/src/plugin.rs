use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unevents_core::events::loadlevel::{
    LevelLoadedEvent, LevelReadyEvent, LoadLevelEvent, MapEntitiesReadyEvent,
    MapGeometryInitializedEvent,
};
use unmapload_core::assets::MapAssets;
use untypes_core::states::AppState;

/// Plugin for map loading functionality
///
/// This plugin registers all the necessary systems and resources for map loading,
/// including level setup, entity spawning, and post-processing.
pub struct UnhaunterMapLoadPlugin;

impl Plugin for UnhaunterMapLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(LoadingState::new(AppState::Loading).load_collection::<MapAssets>());
        app.add_message::<LoadLevelEvent>()
            .add_message::<LevelLoadedEvent>()
            .add_message::<LevelReadyEvent>()
            .add_message::<MapGeometryInitializedEvent>()
            .add_message::<MapEntitiesReadyEvent>();

        // Call the main app_setup from the module
        crate::module::app_setup(app);
    }
}
