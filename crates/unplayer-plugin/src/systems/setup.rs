use bevy::prelude::*;
use untypes_core::states::{AppState, GameState, SimulationState};

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

pub(crate) fn app_setup_core(app: &mut App) {
    hydration::app_setup(app);
    grabdrop::app_setup(app);

    // Configure the authoritative logic set
    app.configure_sets(
        Update,
        unplayer_core::authoritative::PlayerAuthoritativeLogicSet
            .run_if(resource_exists::<untypes_core::roles::AuthorityRole>)
            .after(unplayer_core::PlayerInputSet),
    );

    // Interaction request handling (Authoritative on Host)
    app.add_systems(
        Update,
        (
            // Interaction system runs before movement (Runs on all instances)
            movement::player_interaction_system,
            // Movement system runs after input and waypoints
            // On the client, it only runs for the MainPlayer. On the host, it runs for all players.
            movement::player_movement_system,
        )
            .chain()
            .after(unplayer_core::PlayerInputSet)
            .after(unplayer_core::authoritative::PlayerAuthoritativeLogicSet)
            .run_if(in_state(SimulationState::Ready)),
    );

    app.add_systems(
        PostUpdate,
        input::keyboard::player_input_clear_system.run_if(in_state(SimulationState::Ready)),
    );

    // Gear toggle system must run on all instances (including dedicated server)
    // so that the host can process toggle requests from clients.
    app.add_systems(
        Update,
        input::mouse_interaction::player_gear_usage_system
            .in_set(unplayer_core::PlayerInputSet)
            .run_if(in_state(AppState::InGame)),
    );

    sanityhealth::app_setup(app);
}

pub(crate) fn app_setup_client(app: &mut App) {
    hide::app_setup(app);

    app.add_systems(
        Update,
        styling::update_player_styling.run_if(in_state(AppState::InGame)),
    );

    // Set up input and movement systems with proper ordering
    app.add_systems(
        Update,
        (
            // Input systems run first (Always run on all instances to gather input)
            input::keyboard::keyboard_input_system,
            // Walk target indicator system (kept for compatibility)
            walk_target_indicator::manage_walk_target_indicator,
            // Mouse interaction systems (gear only, clicks handled by waypoint system)
            input::mouse_interaction::mouse_scroll_gear_system,
            input::mouse_interaction::mouse_over_interactive_system,
            input::mouse_interaction::mouse_out_interactive_system,
            // Waypoint systems handle all click-to-move and click-to-interact
            waypoint::waypoint_creation_system,
            waypoint::remote_player_waypoint_system,
            waypoint::waypoint_following_system,
            waypoint::waypoint_queue_cleanup_system,
        )
            .chain()
            .in_set(unplayer_core::PlayerInputSet)
            .run_if(in_state(AppState::InGame)),
    );

    app.add_systems(
        Update,
        // Stairs system runs last. Also gated similarly.
        keyboard::stairs_player
            .after(unplayer_core::PlayerInputSet)
            .run_if(in_state(AppState::InGame)),
    );

    app.add_systems(
        Update,
        // Sync viewer data for rendering
        viewer_sync::viewer_visual_sync.run_if(
            in_state(AppState::InGame).and(
                in_state(GameState::None)
                    .or(in_state(GameState::Truck))
                    .or(in_state(GameState::NpcHelp))
                    .or(in_state(GameState::Pause)),
            ),
        ),
    );

    mouse::app_setup(app);
}
