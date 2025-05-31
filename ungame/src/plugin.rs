use crate::evidence_perception;
use crate::{boardfield_update, hide_mouse, looking_gear, systems};

use super::{game_ui, object_charge, pause_ui, roomchanged};
use bevy::prelude::*;
use uncore::components::game_config::GameConfig;

pub struct UnhaunterGamePlugin;

impl Plugin for UnhaunterGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>();

        systems::app_setup(app);
        hide_mouse::app_setup(app);
        boardfield_update::app_setup(app);
        game_ui::app_setup(app);
        roomchanged::app_setup(app);
        pause_ui::app_setup(app);
        object_charge::app_setup(app);
        evidence_perception::app_setup(app);
        looking_gear::app_setup(app);
    }
}
