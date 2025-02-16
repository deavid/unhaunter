use bevy::prelude::*;
// use uncore::resources::board_data::BoardData;

use crate::resources::MiasmaConfig;

pub struct UnhaunterFogPlugin;

impl Plugin for UnhaunterFogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MiasmaConfig>();

        //.add_systems(OnEnter(AppState::InGame), systems::initialize_miasma)
        // .add_systems(FixedUpdate, systems::update_miasma.run_if(in_state(AppState::InGame)));
        // Removed systems for now.
    }
}
