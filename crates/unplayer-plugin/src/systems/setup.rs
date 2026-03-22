use bevy::prelude::*;
use untypes_core::states::AppState;

use crate::systems::hide;
use crate::systems::hydration;
use crate::systems::input;
use crate::systems::styling;
use crate::systems::walk_target_indicator;
use crate::systems::waypoint;

pub(crate) fn app_setup_core(app: &mut App) {
    hydration::app_setup(app);

    // Configure the authoritative logic set
    app.configure_sets(
        Update,
        unplayer_core::authoritative::PlayerAuthoritativeLogicSet
            .run_if(resource_exists::<untypes_core::roles::AuthorityRole>)
            .after(uninput_core::PlayerInputSet),
    );

    // Gear toggle system must run on all instances (including dedicated server)
    // so that the host can process toggle requests from clients.
    app.add_systems(
        Update,
        input::mouse_interaction::toggle_gear_from_use_intent
            .in_set(unplayer_core::authoritative::PlayerAuthoritativeLogicSet)
            .run_if(in_state(AppState::InGame)),
    );
}

pub(crate) fn app_setup_client(app: &mut App) {
    hide::app_setup(app);

    app.add_systems(
        Update,
        styling::apply_player_tint_color.run_if(in_state(AppState::InGame)),
    );

    // Set up input and movement systems with proper ordering
    app.add_systems(
        Update,
        (
            // Waypoint systems handle all click-to-move and click-to-interact
            waypoint::create_waypoints_from_click,
            waypoint::resolve_movement_from_waypoints,
            waypoint::prune_stale_waypoints,
        )
            .chain()
            .in_set(uninput_core::PlayerInputSet)
            .run_if(in_state(AppState::InGame)),
    );

    // Walk target indicator system: shows visual feedback for click-to-move target
    app.add_systems(
        Update,
        walk_target_indicator::update_move_target_indicator
            .run_if(in_state(AppState::InGame)),
    );

    app.add_systems(
        Update,
        // Stairs system runs last. Also gated similarly.
        crate::systems::keyboard::adjust_elevation_on_stairs
            .after(unplayer_core::authoritative::PlayerAuthoritativeLogicSet)
            .run_if(in_state(AppState::InGame)),
    );
}
