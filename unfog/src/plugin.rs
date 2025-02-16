use bevy::prelude::*;
use uncore::{events::loadlevel::LevelReadyEvent, states::AppState};
// use uncore::resources::board_data::BoardData;

use crate::{resources::MiasmaConfig, systems};

pub struct UnhaunterFogPlugin;

impl Plugin for UnhaunterFogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MiasmaConfig>()
            .add_systems(
                Update,
                systems::initialize_miasma.run_if(on_event::<LevelReadyEvent>),
            )
            .add_systems(Update, systems::spawn_miasma)
            .add_systems(
                FixedUpdate,
                systems::animate_miasma_sprites.run_if(in_state(AppState::InGame)),
            );
        // .add_systems(FixedUpdate, systems::update_miasma.run_if(in_state(AppState::InGame)));
        // Removed systems for now.
    }
}
