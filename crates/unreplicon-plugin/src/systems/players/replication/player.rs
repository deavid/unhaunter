use bevy::prelude::*;
use bevy_replicon::prelude::AppMarkerExt;
use unplayer_core::components::PlayerSprite;
use unreplicon_core::noop::{noop_remove, noop_write};
use unreplicon_core::ownership::LocallyOwned;

pub(super) fn register_locally_owned_marker(app: &mut App) {
    app.set_marker_fns::<LocallyOwned, PlayerSprite>(noop_write::<PlayerSprite>, noop_remove);
}
