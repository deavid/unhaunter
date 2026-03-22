use bevy::prelude::*;
use bevy_replicon::prelude::AppMarkerExt;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::{HeldObject, PlayerGear};
use ungear_core::resources::spawner::GearMarker;
use ungear_core::types::gear::kind::GearKind;
use unreplicon_core::ownership::LocallyOwned;

pub(super) fn register_locally_owned_marker(app: &mut App) {
    app.set_marker_fns::<LocallyOwned, PlayerGear>(
        super::super::noop_write::<PlayerGear>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, HeldObject>(
        super::super::noop_write::<HeldObject>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, GearMarker>(
        super::super::noop_write::<GearMarker>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, GearKind>(
        super::super::noop_write::<GearKind>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, DeployedGear>(
        super::super::noop_write::<DeployedGear>,
        super::super::noop_remove,
    );
}
