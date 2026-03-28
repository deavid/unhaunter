mod perspective;

use bevy::prelude::*;

pub(crate) fn app_setup(app: &mut App) {
    perspective::app_setup(app);
}
