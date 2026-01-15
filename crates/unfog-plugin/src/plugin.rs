use bevy::prelude::*;

use crate::metrics;
use unfog_core::miasma::MiasmaGrid;
use unfog_core::resources::MiasmaConfig;

pub struct UnhaunterFogPlugin;

impl Plugin for UnhaunterFogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MiasmaConfig>()
            .init_resource::<MiasmaGrid>();

        crate::systems::app_setup(app);

        metrics::register_all(app);
    }
}
