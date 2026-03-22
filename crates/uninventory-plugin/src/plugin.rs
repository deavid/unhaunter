use bevy::prelude::*;
use crate::systems;

pub struct UnhaunterInventoryPlugin;

impl Plugin for UnhaunterInventoryPlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
    }
}
