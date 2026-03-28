use bevy::prelude::*;
use uncommon_app_core::states::{AppState, SimulationState};

pub fn simulation_state_transitions(
    app_state: Res<State<AppState>>,
    sim_state: Res<State<SimulationState>>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
) {
    if *app_state.get() == AppState::InGame && *sim_state.get() == SimulationState::Spawning {
        next_sim_state.set(SimulationState::Ready);
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, simulation_state_transitions);
}
