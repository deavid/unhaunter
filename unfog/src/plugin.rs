use bevy::prelude::*;

use crate::{metrics, resources::MiasmaConfig};

pub struct UnhaunterFogPlugin;

impl Plugin for UnhaunterFogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MiasmaConfig>();

        crate::systems::app_setup(app);

        metrics::register_all(app);
    }
}
