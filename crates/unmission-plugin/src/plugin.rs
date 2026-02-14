use bevy::prelude::*;

use crate::systems;

pub struct UnhaunterMissionPlugin;

impl Plugin for UnhaunterMissionPlugin {
    fn build(&self, app: &mut App) {
        systems::setup::app_setup(app);
    }
}
