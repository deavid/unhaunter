use bevy::prelude::*;

pub struct UnhaunterCorePlugin;

impl Plugin for UnhaunterCorePlugin {
    fn build(&self, app: &mut App) {
        crate::metric_recorder::app_setup(app);
    }
}
