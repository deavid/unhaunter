use crate::systems;
use bevy::prelude::*;

pub struct UnhaunterCareerPlugin;

impl Plugin for UnhaunterCareerPlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
    }
}
