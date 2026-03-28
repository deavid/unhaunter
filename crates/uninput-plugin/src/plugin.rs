use crate::systems;
use bevy::prelude::*;
use uninput_core::resources::MissionInputFocus;
use uninput_core::states::InGameUiState;
use unmission_core::types::SimulationState;
use unorchestrator_core::UIContextState;

pub struct UnhaunterInputPlugin;

impl Plugin for UnhaunterInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MissionInputFocus>();
        app.init_state::<InGameUiState>();
        app.add_systems(OnEnter(UIContextState::InGame), enter_running_mode);

        app.add_systems(
            Update,
            (
                systems::keyboard::keyboard_input_system,
                systems::mouse::mouse_aim_system,
                systems::mouse_interaction::mouse_scroll_gear_system,
            )
                .in_set(uninput_core::PlayerInputSet)
                .run_if(in_state(UIContextState::InGame)),
        );

        app.add_systems(
            PostUpdate,
            clear_transient_input_flags.run_if(in_state(SimulationState::Ready)),
        );
        systems::cursor::app_setup(app);
        systems::focus::app_setup(app);
    }
}

fn clear_transient_input_flags(mut q_input: Query<&mut uninput_core::components::PlayerInput>) {
    for mut input in q_input.iter_mut() {
        input.clear();
    }
}

fn enter_running_mode(mut next_game_state: ResMut<NextState<InGameUiState>>) {
    next_game_state.set(InGameUiState::Running);
}
