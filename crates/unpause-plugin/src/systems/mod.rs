mod pause;

use bevy::prelude::*;

pub(crate) fn app_setup(app: &mut App) {
    pause::app_setup(app);
}
