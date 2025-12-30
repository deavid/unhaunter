use bevy::prelude::*;

use crate::events;

/// Plugin that adds all menu component systems to the app
pub struct UnhaunterCoreMenuPlugin;

impl Plugin for UnhaunterCoreMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<events::MenuItemClicked>()
            .add_message::<events::MenuItemSelected>()
            .add_message::<events::MenuEscapeEvent>()
            .add_message::<events::KeyboardNavigate>();

        crate::systems::app_setup(app);
        crate::scrollbar::app_setup(app);
    }
}
