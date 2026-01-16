use super::systems;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unplayer_core::assets::PlayerAssets;
use unplayer_core::resources::PlayerState;
use untypes_core::states::AppState;

pub struct UnhaunterPlayerPlugin;

impl Plugin for UnhaunterPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::Loading).load_collection::<PlayerAssets>(),
        );
        app.init_resource::<PlayerState>();
        systems::app_setup(app);
    }
}
