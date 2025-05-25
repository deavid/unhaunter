use bevy::prelude::*;

use crate::events::KeyboardNavigate;
use crate::systems;

/// Plugin that adds all menu component systems to the app
pub struct UnhaunterCoreMenuPlugin;

impl Plugin for UnhaunterCoreMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<systems::MenuItemClicked>()
            .add_event::<systems::MenuItemSelected>()
            .add_event::<systems::MenuEscapeEvent>()
            .add_event::<KeyboardNavigate>();

        crate::systems::app_setup(app);
        crate::scrollbar::app_setup(app);
    }
}
