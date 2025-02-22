use bevy::prelude::*;

pub struct UnhaunterCorePlugin;

impl Plugin for UnhaunterCorePlugin {
    fn build(&self, app: &mut App) {
        crate::metrics::app_setup(app);
    }
}
