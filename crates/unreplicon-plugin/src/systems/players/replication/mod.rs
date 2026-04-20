use bevy::prelude::*;
use bevy_replicon::prelude::AppMarkerExt;
use bevy_replicon::prelude::AppRuleExt;
use unreplicon_core::ownership::LocallyOwned;
use unreplicon_core::ownership::Owner;

pub(super) fn app_setup(app: &mut App) {
    // Owner is a networking primitive owned by unreplicon.
    // Position replication is registered in unlocomotion-plugin.
    app.replicate::<Owner>();

    // Register LocallyOwned as a receive marker to shield client-driven components.
    // NOTE: Owner is intentionally NOT shielded — it is strictly server-authoritative identity.
    // Shielding it would cause clients to miss ownership transfers when gear changes hands.
    app.register_marker::<LocallyOwned>();
}
