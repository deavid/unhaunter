use bevy::prelude::*;
use bevy_persistent::Persistent;
use unboard_core::resources::board_topology::BoardTopology;
use unevents_core::events::mission::MissionEvent;
use unprofile_core::profile::PlayerProfileData;
use unreplicon_core::components::{LobbyInfo, ServerGamePhase};
use unsummary_core::summary::SummaryData;
use untypes_core::states::{GameState, SimulationState};

pub(crate) fn handle_mission_events(
    mut ev_mission: MessageReader<MissionEvent>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
    mut game_next_state: ResMut<NextState<GameState>>,
    mut o_player_profile: Option<ResMut<Persistent<PlayerProfileData>>>,
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
                let mut initial_deposit_held = 0;
                if let Some(player_profile) = o_player_profile.as_mut() {
                    initial_deposit_held = player_profile.progression.insurance_deposit;

                    player_profile.progression.bank += initial_deposit_held;
                    player_profile.progression.insurance_deposit = 0;

                    if let Err(e) = player_profile.persist() {
                        warn!("Failed to persist PlayerProfileData: {:?}", e);
                    }
                }

                if let Some(summary_data) = summary_data.as_mut() {
                    // Set summary_data.current_mission_id from board_topology.map_path
                    summary_data.map_path = board_topology.map_path.clone();

                    summary_data.deposit_originally_held = initial_deposit_held;
                    summary_data.deposit_returned_to_bank = initial_deposit_held;
                    summary_data.costs_deducted_from_deposit = 0;
                    summary_data.money_earned = 0;

                    if summary_data.ghosts_unhaunted == summary_data.ghost_types.len() as u32 {
                        // All ghosts were unhaunted, successful completion
                        summary_data.mission_successful = true;
                    } else {
                        summary_data.mission_successful = false;
                    }
                }

                game_next_state.set(GameState::Running);
                next_sim_state.set(SimulationState::TearingDown);

                for (mut phase, mut lobby) in q_server_phase.iter_mut() {
                    *phase = ServerGamePhase::Concluding;
                    lobby.set_changed();
                }
            }
        }
    }
}
