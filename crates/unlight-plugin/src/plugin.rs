use bevy::prelude::*;

use crate::{audio, lighting_sim, maplight, metrics, resources::light_grid::LightGrid};

pub struct UnhaunterLightPlugin;

impl Plugin for UnhaunterLightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightGrid>()
            .add_systems(PostUpdate, lighting_sim::systems::rebuild_lighting_field);
        audio::app_setup(app);
        maplight::app_setup(app);
        metrics::register_all(app);
    }
}
