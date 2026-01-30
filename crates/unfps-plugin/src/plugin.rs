use bevy::prelude::*;
use unfps_core::resources::{FpsLimit, FpsLimitRemaining, FpsLimitUsage};

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
