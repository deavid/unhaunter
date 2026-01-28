use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unevents_core::events::sound::SoundEvent;
use unplayer_core::resources::game_config::GameConfig;
use untypes_core::states::AppState;

use super::systems;
use ungear_core::assets::GearAssets;
use ungear_core::resources::spawner::GearSpawnerRegistry;

pub struct UnhaunterGearPlugin;

impl Plugin for UnhaunterGearPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(LoadingState::new(AppState::Loading).load_collection::<GearAssets>());
        app.init_resource::<GameConfig>()
            .init_resource::<GearSpawnerRegistry>()
            .add_message::<SoundEvent>();

        systems::app_setup(app);
    }
}
