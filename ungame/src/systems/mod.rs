pub mod game_systems;
pub mod hint_acknowledge_system;
pub mod hint_ui_system;

use bevy::prelude::App;

pub(crate) fn app_setup(app: &mut App) {
    game_systems::app_setup(app);
    hint_acknowledge_system::app_setup(app);
    hint_ui_system::app_setup(app);
}
