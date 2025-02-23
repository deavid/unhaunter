use bevy::prelude::*;
use uncore::{events::loadlevel::LevelReadyEvent, states::AppState};

use crate::{
    metrics,
    resources::{MiasmaConfig, PerlinNoiseTable},
    systems,
};

pub struct UnhaunterFogPlugin;

impl Plugin for UnhaunterFogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MiasmaConfig>()
            .init_resource::<PerlinNoiseTable>()
            .add_systems(
                Update,
                systems::initialize_miasma.run_if(on_event::<LevelReadyEvent>),
            )
            .add_systems(Update, systems::spawn_miasma)
            .add_systems(
                Update,
                (systems::animate_miasma_sprites, systems::update_miasma)
                    .run_if(in_state(AppState::InGame)),
            );
        metrics::register_all(app);
    }
}
