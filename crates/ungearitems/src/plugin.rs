use bevy::prelude::*;
use unplayer_core::GameConfig;

use crate::metrics;

pub struct UnhaunterGearItemsPlugin;

impl Plugin for UnhaunterGearItemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>();

        crate::components::quartz::app_setup(app);
        crate::components::salt::app_setup(app);
        crate::components::sage::app_setup(app);
        crate::components::thermometer::app_setup(app);
        crate::components::recorder::app_setup(app);
        crate::components::repellentflask::app_setup(app);

        metrics::register_all(app);
    }
}
