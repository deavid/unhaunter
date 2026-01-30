use bevy::prelude::*;

use unmenu_core::events;
use unmenu_core::mission_select::CurrentMissionSelectMode;

/// Plugin that adds all menu component systems to the app
pub struct UnhaunterCoreMenuPlugin;

impl Plugin for UnhaunterCoreMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentMissionSelectMode>();
        app.add_message::<events::MenuItemClicked>()
            .add_message::<events::MenuItemSelected>()
            .add_message::<events::MenuEscapeEvent>()
            .add_message::<events::KeyboardNavigate>();

        crate::systems::app_setup(app);
        crate::scrollbar::app_setup(app);
    }
}
