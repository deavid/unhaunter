use bevy::app::App;

use crate::systems::{client_in_truck_manager, truck_ui_systems};

pub(crate) fn app_setup(app: &mut App) {
    client_in_truck_manager::app_setup(app);
    truck_ui_systems::app_setup(app);
}
