use super::systems;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unorchestrator_core::UIContextState;
use unplayer_core::assets::PlayerAssets;

pub struct UnhaunterPlayerCorePlugin;

impl Plugin for UnhaunterPlayerCorePlugin {
    fn build(&self, app: &mut App) {
        systems::setup::app_setup_core(app);
    }
}

pub struct UnhaunterPlayerPlugin;

impl Plugin for UnhaunterPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(UIContextState::EngineBoot).load_collection::<PlayerAssets>(),
        );
        systems::setup::app_setup_client(app);
    }
}
