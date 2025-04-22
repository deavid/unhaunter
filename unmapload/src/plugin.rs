use bevy::prelude::*;
use uncore::events::loadlevel::{LevelLoadedEvent, LevelReadyEvent, LoadLevelEvent};

/// Plugin for map loading functionality
///
/// This plugin registers all the necessary systems and resources for map loading,
/// including level setup, entity spawning, and post-processing.
pub struct UnMapLoadPlugin;

impl Plugin for UnMapLoadPlugin {
    fn build(&self, app: &mut App) {
        // This is essentially the same logic as the previous app_setup
        // but encapsulated in a proper plugin
        app.add_event::<LoadLevelEvent>()
            .add_event::<LevelLoadedEvent>()
            .add_event::<LevelReadyEvent>()
            .add_systems(PostUpdate, crate::module::load_level_handler)
            .add_systems(
                Update,
                (
                    crate::module::process_pre_meshes,
                    crate::module::after_level_ready,
                ),
            )
            .add_systems(
                Update,
                crate::module::load_map_add_prebaked_lighting.run_if(on_event::<LevelReadyEvent>),
            )
            .add_systems(
                Update,
                crate::influence_system::assign_ghost_influence_system,
            );
    }
}
