use bevy::prelude::*;

use crate::events::KeyboardNavigate;
use crate::systems;

/// Plugin that adds all menu component systems to the app
pub struct UnhaunterCoreMenuPlugin;

impl Plugin for UnhaunterCoreMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<systems::MenuItemClicked>()
            .add_message::<systems::MenuItemSelected>()
            .add_message::<systems::MenuEscapeEvent>()
            .add_message::<KeyboardNavigate>();

        crate::systems::app_setup(app);
        crate::scrollbar::app_setup(app);
    }
}
