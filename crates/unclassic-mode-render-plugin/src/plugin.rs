use bevy::prelude::*;

pub struct ClassicModeRenderPlugin;

impl Plugin for ClassicModeRenderPlugin {
    fn build(&self, app: &mut App) {
        crate::cleanup::app_setup(app);
        crate::hydration::app_setup(app);
        crate::camera::app_setup(app);
        crate::player_names::app_setup(app);
    }
}
