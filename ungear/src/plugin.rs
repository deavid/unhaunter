use bevy::prelude::*;
use uncore::components::game_config::GameConfig;
use uncore::events::sound::SoundEvent;

use super::systems;

pub struct UnhaunterGearPlugin;

impl Plugin for UnhaunterGearPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>().add_event::<SoundEvent>();

        systems::app_setup(app);
    }
}
