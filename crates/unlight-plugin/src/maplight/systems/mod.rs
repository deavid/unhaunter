pub(crate) mod gathering;
pub(crate) mod sprites;
pub(crate) mod tiles;

use crate::maplight::definitions::ActiveFlashlights;
use bevy::prelude::*;

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<ActiveFlashlights>();
}
