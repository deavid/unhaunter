use bevy::prelude::*;

use crate::{bevy::MapTileSetDb, init_maps::init_maps};

pub struct UnhaunterTmxMapPlugin;

impl Plugin for UnhaunterTmxMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_maps)
        .init_resource::<MapTileSetDb>();
    }
}
