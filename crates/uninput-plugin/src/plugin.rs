use crate::systems;
use bevy::prelude::*;
use uncommon_app_core::states::{AppState, SimulationState};
use uninput_core::resources::MissionInputFocus;

pub struct UnhaunterInputPlugin;

impl Plugin for UnhaunterInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MissionInputFocus>();

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
        systems::cursor::app_setup(app);
        systems::focus::app_setup(app);
    }
}

fn clear_transient_input_flags(mut q_input: Query<&mut uninput_core::components::PlayerInput>) {
    for mut input in q_input.iter_mut() {
        input.clear();
    }
}
