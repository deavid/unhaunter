use bevy::prelude::*;
use uncore::components::game_config::GameConfig;
use uncore::events::sound::SoundEvent;
use uncore::states::GameState;

use super::systems;

pub struct UnhaunterGearPlugin;

impl Plugin for UnhaunterGearPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .add_systems(FixedUpdate, systems::update_playerheld_gear_data)
            .add_systems(FixedUpdate, systems::update_deployed_gear_data)
            .add_systems(FixedUpdate, systems::update_deployed_gear_sprites)
            .add_systems(FixedUpdate, systems::update_gear_ui)
            .add_systems(
                Update,
                systems::keyboard_gear.run_if(in_state(GameState::None)),
            )
            .add_systems(Update, systems::sound_playback_system)
            .add_event::<SoundEvent>();
    }
}
