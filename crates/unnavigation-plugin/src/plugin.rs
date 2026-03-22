use super::systems;
use bevy::prelude::*;

pub struct UnhaunterNavigationPlugin;

impl Plugin for UnhaunterNavigationPlugin {
    fn build(&self, app: &mut App) {
        systems::setup::app_setup(app);
    }
}
