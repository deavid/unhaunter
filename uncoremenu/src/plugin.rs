use bevy::prelude::*;

use crate::systems;

/// Plugin that adds all menu component systems to the app
pub struct UnhaunterCoreMenuPlugin;

impl Plugin for UnhaunterCoreMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<systems::MenuItemClicked>()
            .add_event::<systems::MenuItemSelected>()
            .add_event::<systems::MenuEscapeEvent>()
            .add_systems(
                Update,
                (
                    systems::menu_mouse_movement_system,
                    systems::menu_interaction_system,
                    systems::menu_keyboard_system,
                    systems::update_menu_item_visuals,
                )
                    .chain(),
            );
    }
}
