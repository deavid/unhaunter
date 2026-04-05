use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use uncommon_states_core::UIContextState;

use crate::assets::PauseAssets;
use crate::systems;

pub struct UnpausePlugin;

impl Plugin for UnpausePlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(UIContextState::EngineBoot).load_collection::<PauseAssets>(),
        );
        systems::app_setup(app);
    }
}
