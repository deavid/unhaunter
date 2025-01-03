use bevy::prelude::*;
use uncore::components::game_config::GameConfig;
use uncore::events::sound::SoundEvent;

use super::systems;
use super::ui;
use crate::gear_items::*;

pub struct UnhaunterGearPlugin;

impl Plugin for UnhaunterGearPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .add_systems(FixedUpdate, systems::update_playerheld_gear_data)
            .add_systems(FixedUpdate, systems::update_deployed_gear_data)
            .add_systems(FixedUpdate, systems::update_deployed_gear_sprites)
            .add_systems(Update, quartz::update_quartz_and_ghost)
            .add_systems(Update, salt::salt_particle_system)
            .add_systems(Update, salt::salt_pile_system)
            .add_systems(Update, salt::salty_trace_system)
            .add_systems(Update, sage::sage_smoke_system)
            .add_systems(Update, thermometer::temperature_update)
            .add_systems(Update, recorder::sound_update)
            .add_systems(Update, repellentflask::repellent_update)
            .add_systems(Update, systems::sound_playback_system)
            .add_event::<SoundEvent>();
        ui::app_setup(app);
    }
}
