use super::systems;
use bevy::prelude::*;

pub struct UnhaunterNavigationPlugin;

pub struct UnhaunterNavigationClientPlugin;

impl Plugin for UnhaunterNavigationPlugin {
    fn build(&self, app: &mut App) {
        systems::setup::app_setup_core(app);
    }
}

impl Plugin for UnhaunterNavigationClientPlugin {
    fn build(&self, app: &mut App) {
        systems::setup::app_setup_client(app);
    }
}
