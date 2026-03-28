use bevy::app::App;

use crate::systems::truck_ui_systems;

pub(crate) fn app_setup(app: &mut App) {
    truck_ui_systems::app_setup(app);
}
