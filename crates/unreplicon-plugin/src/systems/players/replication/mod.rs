use bevy::prelude::*;
use bevy_replicon::prelude::AppMarkerExt;
use bevy_replicon::prelude::AppRuleExt;
use unreplicon_core::noop::{noop_remove, noop_write};
use unreplicon_core::ownership::LocallyOwned;
use unreplicon_core::ownership::Owner;

pub(super) fn app_setup(app: &mut App) {
    // Owner is a networking primitive owned by unreplicon.
    // Position replication is registered in unlocomotion-plugin.
    app.replicate::<Owner>();

    // Register LocallyOwned as a receive marker to shield client-driven components.
    app.register_marker::<LocallyOwned>();
    app.set_marker_fns::<LocallyOwned, Owner>(noop_write::<Owner>, noop_remove);
}
