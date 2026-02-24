use bevy::prelude::*;
use unevents_core::events::mission::MissionEvent;
use unreplicon_core::resources::MissionEndRequested;
use untypes_core::states::AppState;

use crate::systems::evaluate_mission_end;
use crate::systems::handle_mission_events;

pub(crate) fn app_setup(app: &mut App) {
    app.add_message::<MissionEvent>()
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
