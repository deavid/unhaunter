use crate::systems::board::sync_map_entity_field;
use bevy::prelude::*;

/// The core plugin for the Unhaunter game.
pub struct UnhaunterCorePlugin;

impl Plugin for UnhaunterCorePlugin {
    /// Builds the plugin by adding necessary systems to the app.
    fn build(&self, app: &mut App) {
        crate::metric_recorder::app_setup(app);
        app.add_systems(Update, sync_map_entity_field); // Add the system here
    }
}
