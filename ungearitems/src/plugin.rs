use bevy::prelude::*;
use uncore::components::game_config::GameConfig;

use crate::{components::*, metrics};

pub struct UnhaunterGearItemsPlugin;

impl Plugin for UnhaunterGearItemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .add_systems(Update, quartz::update_quartz_and_ghost)
            .add_systems(Update, salt::salt_particle_system)
            .add_systems(Update, salt::salt_pile_system)
            .add_systems(Update, salt::salty_trace_system)
            .add_systems(Update, sage::sage_smoke_system)
            .add_systems(Update, thermometer::temperature_update)
            .add_systems(Update, recorder::sound_update)
            .add_systems(Update, repellentflask::repellent_update);
        metrics::register_all(app);
    }
}
