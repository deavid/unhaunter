use bevy::prelude::*;

use crate::{
    metrics,
    resources::{MiasmaConfig, PerlinNoiseTable},
};

pub struct UnhaunterFogPlugin;

impl Plugin for UnhaunterFogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MiasmaConfig>()
            .init_resource::<PerlinNoiseTable>();

        crate::systems::app_setup(app);

        metrics::register_all(app);
    }
}
