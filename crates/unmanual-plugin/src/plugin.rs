use bevy::prelude::*;

use crate::{manual_logic, preplay_manual_ui, user_manual_ui};
pub struct UnhaunterManualPlugin;

impl Plugin for UnhaunterManualPlugin {
    fn build(&self, app: &mut App) {
        user_manual_ui::app_setup(app);
        preplay_manual_ui::app_setup(app);

        app.insert_resource(manual_logic::create_manual());
    }
}
