use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unmapload_core::assets::{MapAssets, MissionAssets};
use unmapload_core::events::loadlevel::{
    LevelLoadedEvent, LevelReadyEvent, LoadLevelEvent, MapEntitiesReadyEvent,
    MapGeometryInitializedEvent,
};
use untypes_core::cli::CliOptions;
use untypes_core::states::{AppState, BootState, GameState, SimulationState};

/// Plugin for map loading functionality
///
/// This plugin registers all the necessary systems and resources for map loading,
/// including level setup, entity spawning, and post-processing.
pub struct UnhaunterMapLoadPlugin;

impl Plugin for UnhaunterMapLoadPlugin {
    fn build(&self, app: &mut App) {
        let is_headless = app
            .world()
            .get_resource::<CliOptions>()
            .map(|cli| cli.dedicated)
            .unwrap_or(false);

        app.init_state::<AppState>()
            .init_state::<BootState>()
            .init_state::<GameState>()
            .init_state::<SimulationState>()
            .add_loading_state(
                LoadingState::new(AppState::EngineBoot).continue_to_state(AppState::MainMenu),
            );

        if !is_headless {
            app.add_loading_state(
                LoadingState::new(AppState::EngineBoot)
                    .load_collection::<MapAssets>()
                    .load_collection::<MissionAssets>(),
            );
        }
        app.add_message::<LoadLevelEvent>()
            .add_message::<LevelLoadedEvent>()
            .add_message::<LevelReadyEvent>()
            .add_message::<MapGeometryInitializedEvent>()
            .add_message::<MapEntitiesReadyEvent>();

        // Call the main app_setup from the module
        crate::module::app_setup(app);
    }
}
