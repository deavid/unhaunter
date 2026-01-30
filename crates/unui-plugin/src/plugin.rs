use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unevents_core::events::hint::OnScreenHintEvent;
use untypes_core::states::AppState;
use unui_core::assets::UiAssets;

pub struct UnhaunterUiPlugin;

impl Plugin for UnhaunterUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(LoadingState::new(AppState::Loading).load_collection::<UiAssets>());
        app.add_message::<OnScreenHintEvent>();
    }
}
