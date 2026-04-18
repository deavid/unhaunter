use bevy::prelude::*;
use unmission_core::resources::MissionConcludingCinematic;
use unmission_core::types::MissionEvent;
use unmission_core::types::SimulationState;
use unreplicon_core::components::{LobbyInfo, ServerGamePhase};

pub(crate) fn handle_mission_events(
    mut ev_mission: MessageReader<MissionEvent>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
    mut q_server_phase: Query<(&mut ServerGamePhase, &mut LobbyInfo)>,
    mut commands: Commands,
) {
    for ev in ev_mission.read() {
        match ev {
            MissionEvent::End => {
                info!("[MissionEvent::End] Authority entering SimulationState::TearingDown");

                next_sim_state.set(SimulationState::TearingDown);

                if q_server_phase.is_empty() {
                    // TODO(multiplayer-first): remove once offline play uses a local in-memory
                    // transport. With a loopback transport, ServerGamePhase replication would
                    // drive MissionConcludingCinematic insertion the same way as online play.
                    // Offline single-player: no LobbyInfo entity exists, so
                    // on_mission_concluding will never fire. Insert the cinematic directly.
                    info!(
                        "[MissionEvent::End] No LobbyInfo entity — offline single-player, inserting MissionConcludingCinematic directly"
                    );
                    commands.insert_resource(MissionConcludingCinematic {
                        timer: Timer::from_seconds(2.5, TimerMode::Once),
                        inputs_blocked: true,
                    });
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
