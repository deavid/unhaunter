use bevy::prelude::*;

pub struct ClassicModeUiPlugin;

impl Plugin for ClassicModeUiPlugin {
    fn build(&self, app: &mut App) {
        crate::systems::hint_ui_system::app_setup(app);
        crate::game_ui::app_setup(app);
        crate::looking_gear::app_setup(app);
    }
}
