use bevy::prelude::*;
use bevy_replicon::prelude::AppMarkerExt;
use unbehavior_core::components::{FloorItemCollidable, TmxEntityId};
use uninteraction_core::interaction::Toggleable;
use unreplicon_core::ownership::LocallyOwned;

pub(super) fn register_locally_owned_marker(app: &mut App) {
    app.set_marker_fns::<LocallyOwned, TmxEntityId>(
        super::super::noop_write::<TmxEntityId>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, FloorItemCollidable>(
        super::super::noop_write::<FloorItemCollidable>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, Toggleable>(
        super::super::noop_write::<Toggleable>,
        super::super::noop_remove,
    );
}
