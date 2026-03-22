use crate::systems;
use bevy::prelude::*;
use untypes_core::states::{AppState, SimulationState};

pub struct UnhaunterInputPlugin;

impl Plugin for UnhaunterInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::keyboard::keyboard_input_system,
                systems::mouse::mouse_aim_system,
                systems::mouse_interaction::mouse_scroll_gear_system,
            )
                .run_if(in_state(AppState::InGame)),
        );

        app.add_systems(
            PostUpdate,
            clear_transient_input_flags.run_if(in_state(SimulationState::Ready)),
        );
    }
}

fn clear_transient_input_flags(mut q_input: Query<&mut uninput_core::components::PlayerInput>) {
    for mut input in q_input.iter_mut() {
        input.clear();
    }
}
