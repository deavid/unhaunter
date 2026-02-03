use bevy::app::App;

use crate::systems::in_truck_manager;
use crate::systems::journal_blinking_system;
use crate::systems::truck_ui_systems;

pub(crate) fn app_setup(app: &mut App) {
    journal_blinking_system::app_setup(app);
    truck_ui_systems::app_setup(app);
    in_truck_manager::app_setup(app);
}
