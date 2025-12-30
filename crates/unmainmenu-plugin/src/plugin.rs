use bevy::prelude::*;

use crate::mainmenu;

pub struct UnhaunterMenuPlugin;

impl Plugin for UnhaunterMenuPlugin {
    fn build(&self, app: &mut App) {
        mainmenu::app_setup(app);
    }
}
