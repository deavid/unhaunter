use bevy::prelude::*;
use unmission_core::types::SimulationState;

pub fn simulation_state_transitions(mut next_sim_state: ResMut<NextState<SimulationState>>) {
    next_sim_state.set(SimulationState::Ready);
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        simulation_state_transitions.run_if(in_state(SimulationState::Spawning)),
    );
}
