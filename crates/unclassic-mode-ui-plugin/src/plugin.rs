use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use uncommon_states_core::UIContextState;

use crate::assets::GameUiAssets;

pub struct ClassicModeUiPlugin;

impl Plugin for ClassicModeUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(UIContextState::EngineBoot).load_collection::<GameUiAssets>(),
        );
        crate::gear_ui::app_setup(app);
        crate::systems::hint_ui_system::app_setup(app);
        crate::game_ui::app_setup(app);
        crate::looking_gear::app_setup(app);
    }
}
