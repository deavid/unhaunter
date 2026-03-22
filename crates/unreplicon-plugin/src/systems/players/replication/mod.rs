mod boundary;
mod gear;
mod gearitems;
mod player;

use bevy::prelude::*;
use bevy_replicon::prelude::AppMarkerExt;
use bevy_replicon::prelude::AppRuleExt;
use unreplicon_core::ownership::LocallyOwned;
use unreplicon_core::ownership::Owner;
use unspatial_core::position::Position;

pub(super) fn app_setup(app: &mut App) {
    // Owner and Position are networking primitives owned by unreplicon.
    app.replicate::<Owner>();
    app.replicate::<Position>();

    // Register LocallyOwned as a receive marker to shield client-driven components.
    app.register_marker::<LocallyOwned>();
    app.set_marker_fns::<LocallyOwned, Owner>(super::noop_write::<Owner>, super::noop_remove);
    app.set_marker_fns::<LocallyOwned, Position>(super::noop_write::<Position>, super::noop_remove);

    boundary::register_locally_owned_marker(app);
    player::register_locally_owned_marker(app);
    gear::register_locally_owned_marker(app);
    gearitems::register_locally_owned_marker(app);
}
