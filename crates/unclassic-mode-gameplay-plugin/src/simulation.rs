use bevy::prelude::*;
use unmission_core::types::SimulationState;
use unorchestrator_core::UIContextState;

pub fn simulation_state_transitions(
    app_state: Res<State<UIContextState>>,
    sim_state: Res<State<SimulationState>>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
) {
    if *app_state.get() == UIContextState::InGame && *sim_state.get() == SimulationState::Spawning {
        next_sim_state.set(SimulationState::Ready);
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, simulation_state_transitions);
}
