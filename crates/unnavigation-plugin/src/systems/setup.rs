use bevy::prelude::*;
use unorchestrator_core::UIContextState;

use crate::systems::keyboard;
use crate::systems::waypoint;

pub(crate) fn app_setup(app: &mut App) {
    super::spawn::app_setup(app);

    // Set up waypoint and navigation systems with proper ordering
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
            .run_if(in_state(UIContextState::InGame)),
    );

    app.add_systems(
        Update,
        // Stairs system runs last.
        keyboard::adjust_elevation_on_stairs
            .after(unplayer_core::authoritative::PlayerAuthoritativeLogicSet)
            .run_if(in_state(UIContextState::InGame)),
    );
}
