use bevy::prelude::*;
use bevy_persistent::Persistent;
use unboard_core::resources::board_topology::BoardTopology;
use unevents_core::events::mission::MissionEvent;
use unprofile_core::profile::PlayerProfileData;
use unsummary_core::summary::SummaryData;
use untypes_core::states::{AppState, GameState};

pub struct MissionPlugin;

impl Plugin for MissionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MissionEvent>()
            .add_systems(Update, handle_mission_events);
    }
}

pub fn handle_mission_events(
    mut ev_mission: MessageReader<MissionEvent>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_next_state: ResMut<NextState<GameState>>,
    mut player_profile: ResMut<Persistent<PlayerProfileData>>,
    mut summary_data: ResMut<SummaryData>,
    board_topology: Res<BoardTopology>,
) {
    for ev in ev_mission.read() {
        match ev {
            MissionEvent::End => {
                info!(
                    "[MissionEvent::End] Current board_topology.map_path: '{}'",
                    board_topology.map_path
                );

                let initial_deposit_held = player_profile.progression.insurance_deposit;

                player_profile.progression.bank += initial_deposit_held;
                player_profile.progression.insurance_deposit = 0;

                if let Err(e) = player_profile.persist() {
                    warn!("Failed to persist PlayerProfileData: {:?}", e);
                }

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

                game_next_state.set(GameState::None);
                next_state.set(AppState::Summary);
            }
        }
    }
}
