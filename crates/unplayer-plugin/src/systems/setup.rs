use bevy::prelude::*;
use untypes_core::states::{AppState, SimulationState};

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
            .after(unplayer_core::PlayerInputSet),
    );

    app.add_systems(
        PostUpdate,
        clear_transient_input_flags.run_if(in_state(SimulationState::Ready)),
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

pub(crate) fn clear_transient_input_flags(
    mut q_input: Query<&mut unplayer_core::components::PlayerInput>,
) {
    for mut input in q_input.iter_mut() {
        input.clear();
    }
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
            // Walk target indicator system (kept for compatibility)
            walk_target_indicator::update_move_target_indicator,
            // Waypoint systems handle all click-to-move and click-to-interact
            waypoint::create_waypoints_from_click,
            waypoint::rebuild_remote_waypoint_queue,
            waypoint::resolve_movement_from_waypoints,
            waypoint::prune_stale_waypoints,
        )
            .chain()
            .in_set(unplayer_core::PlayerInputSet)
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
