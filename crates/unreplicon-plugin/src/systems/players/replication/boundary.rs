use bevy::prelude::*;
use bevy_replicon::prelude::AppMarkerExt;
use unbehavior_core::components::{FloorItemCollidable, TmxEntityId};
use uninteraction_core::interaction::Toggleable;
use unreplicon_core::ownership::LocallyOwned;

pub(super) fn register_locally_owned_marker(app: &mut App) {
    app.set_marker_fns::<LocallyOwned, TmxEntityId>(
        unreplicon_core::noop::noop_write::<TmxEntityId>,
        unreplicon_core::noop::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, FloorItemCollidable>(
        unreplicon_core::noop::noop_write::<FloorItemCollidable>,
        unreplicon_core::noop::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, Toggleable>(
        unreplicon_core::noop::noop_write::<Toggleable>,
        unreplicon_core::noop::noop_remove,
    );
}
