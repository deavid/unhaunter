use crate::systems;
use bevy::prelude::*;

pub struct UnhaunterInventoryPlugin;

impl Plugin for UnhaunterInventoryPlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
    }
}
