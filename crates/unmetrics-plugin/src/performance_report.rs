use std::time::Duration;

use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use uncommon_app_core::roles::{AuthorityRole, LobbyPresenceRole, LocalPlayerRole};
use uncommon_app_core::states::{AppState, SimulationState};
use uninput_core::states::InGameUiState;

pub fn report_performance(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut timer: Local<ReportTimer>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<InGameUiState>>,
    simulation_state: Res<State<SimulationState>>,
    authority: Option<Res<AuthorityRole>>,
    local_player: Option<Res<LocalPlayerRole>>,
    lobby_presence: Option<Res<LobbyPresenceRole>>,
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
    }
}

pub struct ReportTimer(Timer);

impl Default for ReportTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(5), TimerMode::Repeating))
    }
}
