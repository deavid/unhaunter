use bevy::prelude::*;
use unmapload_core::events::loadlevel::MapEntitiesReadyEvent;

pub struct ClassicModeOrchestratorPlugin;

impl Plugin for ClassicModeOrchestratorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(unmission_core::summary::ActiveMissionEvaluator(Box::new(
            crate::evaluator::ClassicEvaluator,
        )));
        app.add_systems(
            Update,
            crate::orchestrator::classic_mode_orchestrator
                .run_if(on_message::<MapEntitiesReadyEvent>),
        );
    }
}
