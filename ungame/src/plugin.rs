use crate::evidence_perception;
use crate::{
    boardfield_update, hide_mouse::system_hide_mouse, hint_ui_display, looking_gear,
    systems::game_systems, systems::hint_acknowledge_system::acknowledge_blinking_gear_hint_system,
};

use super::{game_ui, object_charge, pause_ui, roomchanged};
use bevy::prelude::*;
use uncore::components::game_config::GameConfig;
use uncore::states::AppState;

pub struct UnhaunterGamePlugin;

impl Plugin for UnhaunterGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .add_systems(OnEnter(AppState::InGame), game_systems::setup)
            .add_systems(OnExit(AppState::InGame), game_systems::cleanup)
            .add_systems(
                Update,
                (
                    game_systems::keyboard,
                    game_systems::keyboard_floor_switch,
                    system_hide_mouse,
                    acknowledge_blinking_gear_hint_system,
                )
                    .run_if(in_state(AppState::InGame)),
            );

        boardfield_update::app_setup(app);
        game_ui::app_setup(app);
        roomchanged::app_setup(app);
        pause_ui::app_setup(app);
        object_charge::app_setup(app);
        looking_gear::app_setup(app);
        evidence_perception::app_setup(app);
        hint_ui_display::app_setup(app);
    }
}
