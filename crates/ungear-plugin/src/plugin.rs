use bevy::prelude::*;
use unevents_core::events::sound::SoundEvent;
use unplayer_core::GameConfig;

use super::systems;
use crate::resources::spawner::GearSpawnerRegistry;

pub struct UnhaunterGearPlugin;

impl Plugin for UnhaunterGearPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .init_resource::<GearSpawnerRegistry>()
            .add_message::<SoundEvent>();

        systems::app_setup(app);
    }
}
