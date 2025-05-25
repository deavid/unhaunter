use bevy::app::App;

pub mod journal_blinking_system;
pub mod truck_ui_systems;
pub mod ui_state_reset_system;

pub(crate) fn app_setup(app: &mut App) {
    journal_blinking_system::app_setup(app);
    truck_ui_systems::app_setup(app);
    ui_state_reset_system::app_setup(app);
}
