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
use crate::systems::sanityhealth;
use crate::systems::styling;
use crate::systems::viewer_sync;
use crate::systems::walk_target_indicator;
use crate::systems::waypoint;

pub(crate) fn app_setup(app: &mut App) {
    use untypes_core::cli::is_host;
    hydration::app_setup(app);
    grabdrop::app_setup(app);
    hide::app_setup(app);

    app.add_systems(
        PostUpdate,
        input::keyboard::player_input_clear_system.run_if(is_host),
    );

    // Set up input and movement systems with proper ordering
    app.add_systems(
        Update,
        (
            // Input systems run first (Always run on all instances to gather input)
            input::keyboard::keyboard_input_system,
            // Gear usage system (authoritative logic)
            input::mouse_interaction::player_gear_usage_system.run_if(is_host),
            // Walk target indicator system (kept for compatibility)
            walk_target_indicator::manage_walk_target_indicator,
            // Mouse interaction systems (gear only, clicks handled by waypoint system)
            input::mouse_interaction::mouse_scroll_gear_system,
            input::mouse_interaction::mouse_over_interactive_system,
            input::mouse_interaction::mouse_out_interactive_system,
            // Waypoint systems handle all click-to-move and click-to-interact
            waypoint::waypoint_creation_system,
            waypoint::waypoint_following_system,
            waypoint::waypoint_queue_cleanup_system,
            // Movement system runs after input and waypoints
            // Gated by is_host: Only the host simulates movement.
            movement::player_movement_system.run_if(is_host),
            // Stairs system runs last
            keyboard::stairs_player.run_if(is_host),
        )
            .chain()
            .run_if(in_state(GameState::None).and(in_state(AppState::InGame))),
    );

    app.add_systems(
        Update,
        // Sync viewer data for rendering
        viewer_sync::viewer_visual_sync.run_if(
            in_state(AppState::InGame).and(
                in_state(GameState::None)
                    .or(in_state(GameState::Truck))
                    .or(in_state(GameState::NpcHelp)),
            ),
        ),
    );

    app.add_systems(
        Update,
        styling::player_style_system.run_if(in_state(AppState::InGame)),
    );

    mouse::app_setup(app);
    sanityhealth::app_setup(app);
}
