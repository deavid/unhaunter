pub(crate) mod animation;
pub(crate) mod execution;
pub(crate) mod selection;
pub(crate) mod visual_effects;

use bevy::prelude::App;

pub(crate) fn app_setup(app: &mut App) {
    selection::app_setup(app);
    execution::app_setup(app);
    animation::app_setup(app);
    visual_effects::app_setup(app);
}
