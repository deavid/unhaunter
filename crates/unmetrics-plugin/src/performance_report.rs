use std::time::Duration;

use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use unreplicon_core::components::{LobbyInfo, ServerGamePhase};
use untypes_core::roles::{AuthorityRole, LobbyPresenceRole, LocalPlayerRole};
use untypes_core::states::{AppState, GameState, SimulationState};

pub fn report_performance(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut timer: Local<ReportTimer>,
    mut game_next_state: ResMut<NextState<GameState>>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    simulation_state: Res<State<SimulationState>>,
    authority: Option<Res<AuthorityRole>>,
    local_player: Option<Res<LocalPlayerRole>>,
    lobby_presence: Option<Res<LobbyPresenceRole>>,
    q_lobby: Query<(&LobbyInfo, Option<&ServerGamePhase>)>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut system_times: Vec<(&str, f64, String)> = Vec::new();

        for diagnostic in diagnostics.iter() {
            if let Some(average) = diagnostic.average() {
                let system_name = diagnostic.path().as_str();
                if system_name.starts_with("un") && system_name.contains("/systems/") {
                    system_times.push((system_name, average, diagnostic.suffix.to_string()));
                }
            }
        }
        // Grab the top-N
        system_times.sort_by_key(|x| ordered_float::OrderedFloat::from(-x.1));
        system_times.truncate(1); // FIXME: Put top 15 back.

        system_times.sort_by_key(|x| x.0);

        let mut total_systems_time = 0.0;
        for (name, time, suffix) in system_times.iter() {
            if *time > 0.05 {
                debug!("{name}: {time:.2} {suffix}");
            }
            if name.starts_with("un") && name.contains("/systems/") {
                total_systems_time += time;
            }
        }
        const MAX_TIME: f64 = 1000.0 / 60.0;
        debug!("systems: {:.2}%", total_systems_time / MAX_TIME * 100.0);
        debug!(
            "App State: {:?} - Game State: {:?} - Simulation: {:?} - AuthorityRole={} LocalPlayerRole={} LobbyPresenceRole={}",
            app_state.get(),
            game_state.get(),
            simulation_state.get(),
            authority.is_some(),
            local_player.is_some(),
            lobby_presence.is_some(),
        );
        for (lobby, phase) in q_lobby.iter() {
            debug!(
                "Lobby: players={} leader={:?} map={:?} phase={:?}",
                lobby.players.len(),
                lobby.leader_uuid,
                lobby.selected_map,
                phase
            );
        }
        if *app_state != AppState::InGame && *game_state != GameState::Running {
            error!(
                "Inconsistent state: AppState: {:?} - GameState: {:?} - setting GameState to None.",
                app_state.get(),
                game_state.get()
            );
            game_next_state.set(GameState::Running);
        }
    }
}

pub struct ReportTimer(Timer);

impl Default for ReportTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(5), TimerMode::Repeating))
    }
}
