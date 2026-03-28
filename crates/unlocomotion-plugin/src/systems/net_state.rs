use bevy::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use unlocomotion_core::components::PlayerLocomotionState;
use unplayer_core::components::PlayerSprite;
use unreplicon_core::export_ext::AppClientExportExt;
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;

pub(crate) fn app_setup(app: &mut App) {
    // Position replication is owned by the locomotion domain (all player entities have Position).
    app.replicate::<Position>();
    app.add_component_export::<Position, PlayerSprite>();
    app.add_component_export::<Direction, PlayerSprite>();
    app.add_component_export::<PlayerLocomotionState, PlayerSprite>();
}
