use crate::systems;
use bevy::prelude::*;

pub struct UnhaunterLocomotionPlugin;

impl Plugin for UnhaunterLocomotionPlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
    }
}
