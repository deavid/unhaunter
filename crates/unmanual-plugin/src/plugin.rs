use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use untypes_core::states::AppState;

use crate::{manual_logic, preplay_manual_ui, user_manual_ui};
use unmanual_core::assets::ManualAssets;
pub struct UnhaunterManualPlugin;

impl Plugin for UnhaunterManualPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::Loading).load_collection::<ManualAssets>(),
        );
        user_manual_ui::app_setup(app);
        preplay_manual_ui::app_setup(app);

        app.insert_resource(manual_logic::create_manual());
    }
}
