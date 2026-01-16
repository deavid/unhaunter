use crate::metrics;
use crate::systems::*;
use bevy::prelude::*;
use unsound_core::resources::SoundGrid;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoundGrid>()
            .add_systems(Update, sound_update)
            .add_systems(Update, init_sound_grid);

        metrics::register_all(app);
    }
}
