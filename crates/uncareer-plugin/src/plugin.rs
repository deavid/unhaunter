use bevy::prelude::*;
use crate::systems;

pub struct UnhaunterCareerPlugin;

impl Plugin for UnhaunterCareerPlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
    }
}
