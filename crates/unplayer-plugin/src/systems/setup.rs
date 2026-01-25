use bevy::prelude::*;
use untypes_core::states::AppState;
use untypes_core::states::GameState;

use crate::systems::grabdrop;
use crate::systems::hide;
use crate::systems::hydration;
use crate::systems::input;
use crate::systems::keyboard;
use crate::systems::mouse;
use crate::systems::movement;
use crate::systems::player_state;
use crate::systems::sanityhealth;
use crate::systems::viewer_sync;
use crate::systems::walk_target_indicator;
use crate::systems::waypoint;

pub(crate) fn app_setup(app: &mut App) {
    hydration::app_setup(app);
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
            // Stairs system runs last
            keyboard::stairs_player,
        )
            .chain()
            .run_if(in_state(GameState::None).and(in_state(AppState::InGame))),
    );

    app.add_systems(
        Update,
        (
            // Update player state for cross-domain access
            player_state::update_player_state,
            // Sync viewer data for rendering
            viewer_sync::viewer_visual_sync,
        )
            .run_if(
                in_state(AppState::InGame).and(
                    in_state(GameState::None)
                        .or(in_state(GameState::Truck))
                        .or(in_state(GameState::NpcHelp)),
                ),
            ),
    );

    mouse::app_setup(app);
    sanityhealth::app_setup(app);
}
