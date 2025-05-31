use bevy::prelude::*;

use crate::{maplight, metrics};

pub struct UnhaunterLightPlugin;

impl Plugin for UnhaunterLightPlugin {
    fn build(&self, app: &mut App) {
        maplight::app_setup(app);
        metrics::register_all(app);
    }
}
