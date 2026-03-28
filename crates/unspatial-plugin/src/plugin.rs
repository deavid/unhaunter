use bevy::prelude::*;

use crate::systems;

/// Plugin that converts logical `Position` components to Bevy screen-space
/// `Transform` components using the isometric perspective projection.
/// Runs in PostUpdate so all game-logic Position mutations from Update are visible.
pub struct UnhaunterSpatialPlugin;

impl Plugin for UnhaunterSpatialPlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
    }
}
