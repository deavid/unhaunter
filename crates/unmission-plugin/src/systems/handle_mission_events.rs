use bevy::prelude::*;
use unmission_core::types::MissionEvent;
use unmission_core::types::SimulationState;
use unreplicon_core::components::{LobbyInfo, ServerGamePhase};

pub(crate) fn handle_mission_events(
    mut ev_mission: MessageReader<MissionEvent>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
    mut q_server_phase: Query<(&mut ServerGamePhase, &mut LobbyInfo)>,
) {
    for ev in ev_mission.read() {
        match ev {
            MissionEvent::End => {
                info!("[MissionEvent::End] Authority entering SimulationState::TearingDown");

                next_sim_state.set(SimulationState::TearingDown);

                if q_server_phase.is_empty() {
                    warn!(
                        "[MissionEvent::End] No LobbyInfo/ServerGamePhase entity found while concluding mission"
                    );
                }

                for (mut phase, mut lobby) in q_server_phase.iter_mut() {
                    if *phase != ServerGamePhase::Concluding {
                        *phase = ServerGamePhase::Concluding;
                        lobby.set_changed();
                    }
                }
            }
        }
    }
}
