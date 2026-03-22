use bevy::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use unmission_core::events::MissionCompletedEvent;
use unmission_core::resources::MissionEndRequested;
use unmission_core::summary::SummaryData;
use unmission_core::types::MissionEvent;
use untypes_core::states::AppState;

use crate::systems::evaluate_mission_end;
use crate::systems::handle_mission_events;

pub(crate) fn app_setup(app: &mut App) {
    app.replicate::<SummaryData>();
    app.add_message::<MissionEvent>()
        .add_message::<MissionCompletedEvent>()
        .init_resource::<MissionEndRequested>()
        .add_systems(
            Update,
            (
                handle_mission_events::handle_mission_events,
                evaluate_mission_end::evaluate_mission_end,
            )
                .run_if(in_state(AppState::InGame)),
        );
}
