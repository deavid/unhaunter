use bevy::prelude::*;
use uninput_core::states::InGameUiState;
use unplayer_core::components::Hiding;
use untruck_core::components::in_truck::InTruck;

fn on_intruck_added(
    trigger: On<Add, InTruck>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<InGameUiState>>,
) {
    commands
        .entity(trigger.entity)
        .insert(Hiding { hiding_spot: None });
    next_state.set(InGameUiState::Truck);
}

fn on_intruck_removed(
    trigger: On<Remove, InTruck>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<InGameUiState>>,
) {
    commands.entity(trigger.entity).remove::<Hiding>();
    next_state.set(InGameUiState::Running);
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_observer(on_intruck_added);
    app.add_observer(on_intruck_removed);
}
