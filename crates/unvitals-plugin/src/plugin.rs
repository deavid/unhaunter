use crate::systems;
use bevy::prelude::*;

pub struct UnhaunterVitalsPlugin;

impl Plugin for UnhaunterVitalsPlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
    }
}
