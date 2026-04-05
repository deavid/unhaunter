use bevy::prelude::App;

use crate::systems::gis::animation;
use crate::systems::gis::execution;
use crate::systems::gis::selection;

pub(crate) fn app_setup(app: &mut App) {
    selection::app_setup(app);
    execution::app_setup(app);
    animation::app_setup(app);
}
