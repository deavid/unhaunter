use bevy::prelude::*;
use uncore::components::game_config::GameConfig;
use uncore::events::sound::SoundEvent;

use super::systems;
use super::ui;

pub struct UnhaunterGearPlugin;

impl Plugin for UnhaunterGearPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .add_systems(FixedUpdate, systems::update_playerheld_gear_data)
            .add_systems(FixedUpdate, systems::update_deployed_gear_data)
            .add_systems(FixedUpdate, systems::update_deployed_gear_sprites)
            .add_systems(Update, systems::sound_playback_system)
            .add_event::<SoundEvent>();
        ui::app_setup(app);
    }
}
