use bevy::prelude::*;

pub struct ClassicModePlugin;

impl Plugin for ClassicModePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(unsummary_core::summary::ActiveMissionEvaluator(Box::new(
            crate::evaluator::ClassicEvaluator,
        )));
        app.add_systems(
            Update,
            (
                crate::systems::classic_mode_orchestrator
                    .run_if(on_message::<unevents_core::events::loadlevel::MapEntitiesReadyEvent>),
                crate::systems::sync_ghost_visuals,
            ),
        );
    }
}
