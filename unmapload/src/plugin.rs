use bevy::prelude::*;
use uncore::events::loadlevel::{LevelLoadedEvent, LevelReadyEvent, LoadLevelEvent};

/// Plugin for map loading functionality
///
/// This plugin registers all the necessary systems and resources for map loading,
/// including level setup, entity spawning, and post-processing.
pub struct UnhaunterMapLoadPlugin;

impl Plugin for UnhaunterMapLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadLevelEvent>()
            .add_event::<LevelLoadedEvent>()
            .add_event::<LevelReadyEvent>();

        // Call the main app_setup from the module
        crate::module::app_setup(app);

        // The influence_system is now also part of the module refactoring.
        crate::influence_system::app_setup(app);
    }
}
