use bevy::prelude::*;

use crate::systems;

pub struct UnrepliconTransportPlugin;

impl Plugin for UnrepliconTransportPlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
    }
}
