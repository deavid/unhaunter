use bevy::prelude::*;

pub mod influence_system;
pub mod selection;
pub mod systems;

pub struct ClassicModePlugin;

impl Plugin for ClassicModePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            systems::classic_mode_orchestrator
                .run_if(on_message::<unevents_core::events::loadlevel::MapEntitiesReadyEvent>),
        );
    }
}
