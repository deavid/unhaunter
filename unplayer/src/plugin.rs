use super::systems;
use bevy::prelude::*;

pub struct UnhaunterPlayerPlugin;

impl Plugin for UnhaunterPlayerPlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
    }
}
