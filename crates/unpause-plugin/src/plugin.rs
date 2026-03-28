use bevy::prelude::*;

use crate::systems;

pub struct UnpausePlugin;

impl Plugin for UnpausePlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
    }
}
