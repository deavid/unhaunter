use crate::resources::{FpsLimit, FpsLimitRemaining, FpsLimitUsage};
use bevy::prelude::*;

pub struct UnhaunterFpsPlugin;

impl Plugin for UnhaunterFpsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FpsLimit>();
        app.init_resource::<FpsLimitRemaining>();
        app.init_resource::<FpsLimitUsage>();
        crate::metrics::register_all(app);
        crate::systems::app_setup(app);
    }
}
