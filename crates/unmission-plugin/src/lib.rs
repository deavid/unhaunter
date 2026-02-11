use bevy::prelude::*;
use bevy_persistent::Persistent;
use unboard_core::resources::board_topology::BoardTopology;
use unevents_core::events::mission::MissionEvent;
use unnet_core::resources::MissionEndRequested;
use unplayer_core::components::PlayerDisconnected;
use unplayer_core::components::PlayerInactive;
use unplayer_core::components::PlayerSpectating;
use unplayer_core::components::PlayerSprite;
use unprofile_core::profile::PlayerProfileData;
use unspatial_core::position::Position;
use unsummary_core::summary::SummaryData;
use untruck_core::components::in_truck::InTruck;
use untypes_core::cli::{CliOptions, NetMode};
use untypes_core::states::{AppState, GameState};

pub struct MissionPlugin;

impl Plugin for MissionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MissionEvent>()
            .init_resource::<MissionEndRequested>()
            .add_systems(Update, (handle_mission_events, evaluate_mission_end));
    }
}

pub fn evaluate_mission_end(
    query_players: Query<
        (
            &Position,
            Has<InTruck>,
            Has<PlayerSpectating>,
            Has<PlayerDisconnected>,
            Has<PlayerInactive>,
        ),
        With<PlayerSprite>,
    >,
    mut ev_mission: MessageWriter<MissionEvent>,
    mut mission_end_requested: ResMut<MissionEndRequested>,
    cli: Res<CliOptions>,
) {
    if !matches!(cli.net_mode, NetMode::Host { .. } | NetMode::Offline) {
        return;
    }

    let mut active_players = 0;
    let mut players_in_truck = 0;
    let mut any_connected = false;

    for (_, in_truck, spectating, disconnected, inactive) in query_players.iter() {
        if disconnected || inactive {
            continue;
        }
        any_connected = true;
        if spectating {
            continue;
        }
        active_players += 1;
        if in_truck {
            players_in_truck += 1;
        }
    }

    let all_in_truck = active_players > 0 && active_players == players_in_truck;

    // Condition A: Update availability for clients/UI
    mission_end_requested.0 = all_in_truck;

    // Condition B: all active players dead (active_players == 0 but at least one player was there)
    if any_connected && active_players == 0 {
        ev_mission.write(MissionEvent::End);
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
