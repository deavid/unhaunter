use bevy::prelude::*;
use unplayer_core::components::PlayerSprite;
use unreplicon_core::export_ext::AppClientExportExt;
use unvitals_core::components::{PlayerVitals, Stamina};

pub(crate) fn app_setup(app: &mut App) {
    app.add_component_export::<Stamina, PlayerSprite>();
    app.add_component_export::<PlayerVitals, PlayerSprite>();
}
