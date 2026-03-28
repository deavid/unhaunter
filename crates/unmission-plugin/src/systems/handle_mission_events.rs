use bevy::prelude::*;
use unboard_core::resources::board_topology::BoardTopology;
use uncommon_app_core::states::SimulationState;
use unmission_core::events::MissionCompletedEvent;
use unmission_core::summary::SummaryData;
use unmission_core::types::MissionEvent;
use unreplicon_core::components::{LobbyInfo, ServerGamePhase};

pub(crate) fn handle_mission_events(
    mut ev_mission: MessageReader<MissionEvent>,
    mut ev_mission_completed: MessageWriter<MissionCompletedEvent>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
    mut summary_data: Option<ResMut<SummaryData>>,
    board_topology: Res<BoardTopology>,
    mut q_server_phase: Query<(&mut ServerGamePhase, &mut LobbyInfo)>,
) {
    for ev in ev_mission.read() {
        match ev {
            MissionEvent::End => {
                info!(
                    "[MissionEvent::End] Current board_topology.map_path: '{}'",
                    board_topology.map_path
                );

                if let Some(summary_data) = summary_data.as_mut() {
                    // Set summary_data.current_mission_id from board_topology.map_path
                    summary_data.map_path = board_topology.map_path.clone();

                    summary_data.deposit_originally_held = 0;
                    summary_data.deposit_returned_to_bank = 0;
                    summary_data.costs_deducted_from_deposit = 0;
                    summary_data.money_earned = 0;

                    if summary_data.ghosts_unhaunted == summary_data.ghost_types.len() as u32 {
                        // All ghosts were unhaunted, successful completion
                        summary_data.mission_successful = true;
                    } else {
                        summary_data.mission_successful = false;
                    }

                    ev_mission_completed.write(MissionCompletedEvent {
                        summary: summary_data.clone(),
                    });
                }

                next_sim_state.set(SimulationState::TearingDown);

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
