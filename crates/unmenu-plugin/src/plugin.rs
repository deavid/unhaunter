use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unmenu_core::assets::MenuAssets;
use unmenu_core::events;
use unmenu_core::mission_select::CurrentMissionSelectMode;
use unorchestrator_core::UIContextState;

/// Plugin that adds all menu component systems to the app
pub struct UnhaunterCoreMenuPlugin;

impl Plugin for UnhaunterCoreMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(UIContextState::EngineBoot).load_collection::<MenuAssets>(),
        );
        app.init_resource::<CurrentMissionSelectMode>();
        app.add_message::<events::MenuItemClicked>()
            .add_message::<events::MenuItemSelected>()
            .add_message::<events::MenuEscapeEvent>()
            .add_message::<events::KeyboardNavigate>();

        crate::systems::app_setup(app);
        crate::scrollbar::app_setup(app);
    }
}
