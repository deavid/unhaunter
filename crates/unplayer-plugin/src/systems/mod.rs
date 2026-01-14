pub(crate) mod grabdrop;
pub(crate) mod hide;
pub(crate) mod input;
pub(crate) mod keyboard;
pub(crate) mod mouse;
pub(crate) mod movement;
pub(crate) mod pathfinding;
pub(crate) mod player_state;
pub(crate) mod sanityhealth;
pub(crate) mod walk_target_indicator;
pub(crate) mod waypoint;

use bevy::prelude::*;
use untypes_core::states::AppState;
use untypes_core::states::GameState;

pub(crate) fn app_setup(app: &mut App) {
    grabdrop::app_setup(app);
    hide::app_setup(app);

    // Set up input and movement systems with proper ordering
    app.add_systems(
        Update,
        (
            // Input systems run first
            input::keyboard::keyboard_input_system,
            // Walk target indicator system (kept for compatibility)
            walk_target_indicator::manage_walk_target_indicator,
            // Mouse interaction systems (gear only, clicks handled by waypoint system)
            input::mouse_interaction::mouse_right_click_gear_system,
            input::mouse_interaction::mouse_scroll_gear_system,
            input::mouse_interaction::mouse_over_interactive_system,
            input::mouse_interaction::mouse_out_interactive_system,
            // Waypoint systems handle all click-to-move and click-to-interact
            waypoint::waypoint_creation_system,
            waypoint::waypoint_following_system,
            waypoint::waypoint_queue_cleanup_system,
            // Movement system runs after input and waypoints
            movement::player_movement_system,
            // Update player state for cross-domain access
            player_state::update_player_state,
            // Stairs system runs last
            keyboard::stairs_player,
        )
            .chain()
            .run_if(in_state(GameState::None).and(in_state(AppState::InGame))),
    );

    mouse::app_setup(app);
    sanityhealth::app_setup(app);
}
