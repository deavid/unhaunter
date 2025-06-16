pub mod grabdrop;
pub mod hide;
pub mod input {
    pub mod keyboard;
    pub mod mouse_interaction;
    pub mod mouse_pathing;
}
pub mod keyboard;
pub mod mouse;
pub mod movement;
pub mod sanityhealth;
pub mod walk_target_indicator;

use bevy::prelude::*;
use uncore::states::AppState;
use uncore::states::GameState;

pub(crate) fn app_setup(app: &mut App) {
    grabdrop::app_setup(app);
    hide::app_setup(app);

    // Set up input and movement systems with proper ordering
    app.add_systems(
        Update,
        (
            // Input systems run first
            input::keyboard::keyboard_input_system,
            input::mouse_pathing::click_to_move_pathing_system,
            input::mouse_pathing::click_to_move_update_system,
            // Walk target indicator system runs after move target updates
            walk_target_indicator::manage_walk_target_indicator,
            // Mouse interaction systems
            input::mouse_interaction::mouse_interaction_system,
            input::mouse_interaction::complete_pending_interaction_system,
            input::mouse_interaction::mouse_right_click_gear_system,
            input::mouse_interaction::mouse_scroll_gear_system,
            input::mouse_interaction::mouse_hover_interactive_system,
            // Movement system runs after input
            movement::player_movement_system,
            // Stairs system runs last
            keyboard::stairs_player,
        )
            .chain()
            .run_if(in_state(GameState::None).and(in_state(AppState::InGame))),
    );

    mouse::app_setup(app);
    sanityhealth::app_setup(app);
}
