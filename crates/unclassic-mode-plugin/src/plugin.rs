use crate::{evidence_perception, game_ui, looking_gear, object_charge, roomchanged};
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
                crate::systems::orchestrator::classic_mode_orchestrator
                    .run_if(on_message::<unevents_core::events::loadlevel::MapEntitiesReadyEvent>),
                crate::systems::orchestrator::sync_ghost_visuals,
            ),
        );

        crate::systems::game_systems::app_setup(app);
        crate::systems::hint_acknowledge_system::app_setup(app);
        crate::systems::hint_ui_system::app_setup(app);
        crate::environmental_mechanics::app_setup(app);

        game_ui::app_setup(app);
        roomchanged::app_setup(app);
        object_charge::app_setup(app);
        evidence_perception::app_setup(app);
        looking_gear::app_setup(app);
    }
}
