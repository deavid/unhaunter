use bevy::prelude::*;
use uncommon_states_core::UIContextState;
use unmission_core::events::QuitMissionEvent;
use unmission_core::resources::MissionEndRequested;
use unmission_core::types::MissionEvent;
use unmission_core::types::SimulationState;

use crate::systems::concluding_cinematic;
use crate::systems::evaluate_mission_end;
use crate::systems::handle_mission_events;
use crate::systems::handle_quit_mission;
use crate::systems::startup;

pub(crate) fn app_setup(app: &mut App) {
    app.add_message::<MissionEvent>()
        .add_message::<QuitMissionEvent>()
        .init_resource::<MissionEndRequested>()
        .init_resource::<startup::PendingMissionStartup>()
        .add_systems(
            Update,
            startup::stage_level_ready
                .run_if(on_message::<unmission_core::events::LevelReadyEvent>),
        )
        .add_systems(
            Update,
            startup::enter_in_game_when_ready
                .run_if(in_state(UIContextState::MissionLoading))
                .run_if(in_state(SimulationState::Ready)),
        )
        .add_systems(
            OnEnter(UIContextState::InGame),
            startup::sync_initial_room_state_on_enter,
        )
        .add_systems(
            OnEnter(UIContextState::Lobby),
            startup::reset_pending_mission_startup,
        )
        .add_systems(
            OnEnter(SimulationState::TearingDown),
            startup::reset_pending_mission_startup,
        )
        .add_systems(
            Update,
            handle_mission_events::handle_mission_events
                .run_if(resource_exists::<unreplicon_core::resources::AuthorityRole>)
                .run_if(in_state(UIContextState::InGame)),
        )
        .add_systems(
            Update,
            evaluate_mission_end::evaluate_mission_end
                .run_if(in_state(UIContextState::InGame))
                .run_if(not(in_state(SimulationState::TearingDown))),
        );
    app.add_systems(
        Update,
        (
            concluding_cinematic::on_mission_concluding,
            concluding_cinematic::tick_mission_concluding,
        )
            .run_if(resource_exists::<unreplicon_core::resources::LocalPlayerRole>)
            .run_if(in_state(UIContextState::InGame)),
    );
    app.add_systems(
        Update,
        handle_quit_mission::handle_quit_mission.run_if(in_state(UIContextState::InGame)),
    );
}
