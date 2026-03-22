use crate::systems;
use bevy::prelude::*;
use untypes_core::states::AppState;

pub struct UnhaunterInputPlugin;

impl Plugin for UnhaunterInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::keyboard::keyboard_input_system,
                systems::mouse::mouse_aim_system,
                systems::mouse_interaction::mouse_scroll_gear_system,
                systems::mouse_interaction::mouse_over_interactive_system,
                systems::mouse_interaction::mouse_out_interactive_system,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}
