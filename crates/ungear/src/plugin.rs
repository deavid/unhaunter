use bevy::prelude::*;
use uncore_events::events::sound::SoundEvent;
use unplayer_core::GameConfig;

use super::systems;

pub struct UnhaunterGearPlugin;

impl Plugin for UnhaunterGearPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .add_message::<SoundEvent>();

        systems::app_setup(app);
    }
}
