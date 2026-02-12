use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unevents_core::events::sound::SoundEvent;
use unplayer_core::resources::game_config::GameConfig;
use untypes_core::states::AppState;

use super::systems;
use crate::metrics;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use unrender_std::assets::GearAssets;

pub struct UnhaunterGearCorePlugin;

impl Plugin for UnhaunterGearCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .init_resource::<GearSpawnerRegistry>()
            .add_message::<SoundEvent>();

        metrics::register_all(app);
    }
}

pub struct UnhaunterGearPlugin;

impl Plugin for UnhaunterGearPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(LoadingState::new(AppState::Loading).load_collection::<GearAssets>());
        systems::app_setup(app);
    }
}
