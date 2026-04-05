use bevy::prelude::*;
use uninput_core::states::InGameUiState;
use untruck_core::components::in_truck::InTruck;

fn on_intruck_added(_trigger: On<Add, InTruck>, mut next_state: ResMut<NextState<InGameUiState>>) {
    next_state.set(InGameUiState::Truck);
}

fn on_intruck_removed(
    _trigger: On<Remove, InTruck>,
    mut next_state: ResMut<NextState<InGameUiState>>,
) {
    next_state.set(InGameUiState::Running);
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_observer(on_intruck_added);
    app.add_observer(on_intruck_removed);
}
