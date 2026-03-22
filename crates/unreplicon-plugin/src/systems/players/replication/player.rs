use bevy::prelude::*;
use bevy_replicon::prelude::AppMarkerExt;
use unlocomotion_core::components::PlayerLocomotionState;
use unplayer_core::components::{Hiding, PlayerSpectating, PlayerSprite};
use unreplicon_core::ownership::LocallyOwned;
use unspatial_core::direction::Direction;
use untruck_core::components::in_truck::InTruck;
use unvitals_core::components::{PlayerVitals, Stamina};

pub(super) fn register_locally_owned_marker(app: &mut App) {
    app.set_marker_fns::<LocallyOwned, Direction>(
        super::super::noop_write::<Direction>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, PlayerSprite>(
        super::super::noop_write::<PlayerSprite>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, PlayerLocomotionState>(
        super::super::noop_write::<PlayerLocomotionState>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, PlayerVitals>(
        super::super::noop_write::<PlayerVitals>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, Stamina>(
        super::super::noop_write::<Stamina>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, Hiding>(
        super::super::noop_write::<Hiding>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, InTruck>(
        super::super::noop_write::<InTruck>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, PlayerSpectating>(
        super::super::noop_write::<PlayerSpectating>,
        super::super::noop_remove,
    );
}
