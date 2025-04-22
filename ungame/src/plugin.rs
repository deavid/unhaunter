use crate::{boardfield_update, hide_mouse::system_hide_mouse, looking_gear};

use super::{game_ui, object_charge, pause_ui, roomchanged, systems};
use bevy::prelude::*;
use uncore::components::game_config::GameConfig;
use uncore::states::AppState;
use unmapload::UnMapLoadPlugin;

pub struct UnhaunterGamePlugin;

impl Plugin for UnhaunterGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .add_systems(OnEnter(AppState::InGame), systems::setup)
            .add_systems(OnExit(AppState::InGame), systems::cleanup)
            .add_systems(
                Update,
                (
                    systems::keyboard,
                    systems::keyboard_floor_switch,
                    system_hide_mouse,
                ),
            )
            .add_plugins(UnMapLoadPlugin);

        boardfield_update::app_setup(app);
        game_ui::app_setup(app);
        roomchanged::app_setup(app);
        pause_ui::app_setup(app);
        object_charge::app_setup(app);
        looking_gear::app_setup(app);
    }
}
