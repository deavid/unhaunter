mod board_sync;
mod boardfield_update;

use bevy::prelude::*;

pub(crate) fn app_setup(app: &mut App) {
    board_sync::register_diagnostics(app);
    board_sync::app_setup(app);
    boardfield_update::app_setup(app);
}
