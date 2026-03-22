use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use untypes_core::states::AppState;

use super::systems;
use crate::metrics;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use unrender_std::assets::GearAssets;

pub struct UnhaunterGearCorePlugin;

impl Plugin for UnhaunterGearCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GearSpawnerRegistry>();

        metrics::register_all(app);
    }
}

pub struct UnhaunterGearPlugin;

impl Plugin for UnhaunterGearPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::EngineBoot).load_collection::<GearAssets>(),
        );
        systems::app_setup(app);
    }
}
