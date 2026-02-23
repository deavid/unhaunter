use bevy::prelude::*;

use crate::systems;

pub struct UnrepliconPlugin;

impl Plugin for UnrepliconPlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
    }
}
