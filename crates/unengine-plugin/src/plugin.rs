use crate::systems;
use bevy::prelude::*;

pub struct UnhaunterEnginePlugin;

impl Plugin for UnhaunterEnginePlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
        crate::pause_ui::app_setup(app);
        crate::hide_mouse::app_setup(app);
        crate::boardfield_update::app_setup(app);
    }
}
