mod boardfield_update;

use bevy::prelude::*;

pub(crate) fn app_setup(app: &mut App) {
    boardfield_update::app_setup(app);
}
