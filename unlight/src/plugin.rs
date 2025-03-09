use bevy::prelude::*;

use crate::{
    maplight::{ambient_sound_system, apply_lighting, player_visibility_system},
    metrics,
};

pub struct UnhaunterLightPlugin;

impl Plugin for UnhaunterLightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (player_visibility_system, apply_lighting).chain())
            .add_systems(Update, ambient_sound_system);
        metrics::register_all(app);
    }
}
