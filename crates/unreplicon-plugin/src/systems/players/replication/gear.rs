use bevy::prelude::*;
use bevy_replicon::prelude::AppMarkerExt;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::{HeldObject, PlayerGear};
use ungear_core::resources::spawner::GearMarker;
use ungear_core::types::gear::kind::GearKind;
use unreplicon_core::ownership::LocallyOwned;

pub(super) fn register_locally_owned_marker(app: &mut App) {
    app.set_marker_fns::<LocallyOwned, PlayerGear>(
        unreplicon_core::noop::noop_write::<PlayerGear>,
        unreplicon_core::noop::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, HeldObject>(
        unreplicon_core::noop::noop_write::<HeldObject>,
        unreplicon_core::noop::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, GearMarker>(
        unreplicon_core::noop::noop_write::<GearMarker>,
        unreplicon_core::noop::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, GearKind>(
        unreplicon_core::noop::noop_write::<GearKind>,
        unreplicon_core::noop::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, DeployedGear>(
        unreplicon_core::noop::noop_write::<DeployedGear>,
        unreplicon_core::noop::noop_remove,
    );
}
