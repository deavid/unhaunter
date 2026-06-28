use bevy::prelude::*;
use uncommon_states_core::UIContextState;
use unmission_core::types::SimulationState;

use crate::metrics;
use unfog_core::resources::MiasmaConfig;

pub struct UnhaunterFogCorePlugin;

impl Plugin for UnhaunterFogCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MiasmaConfig>();

        app.add_systems(Update, crate::systems::init_miasma_grid);
        app.add_systems(
            Update,
            crate::systems::initialize_miasma
                .run_if(bevy::prelude::on_message::<unmission_core::events::LevelReadyEvent>)
                .after(crate::systems::init_miasma_grid),
        );
        app.add_systems(
            Update,
            crate::systems::update_miasma.run_if(in_state(SimulationState::Ready)),
        );
        app.add_systems(
            Update,
            crate::systems::diffuse_smoke_field
                .run_if(in_state(SimulationState::Ready))
                .after(crate::systems::update_miasma),
        );
        app.add_systems(
            Update,
            crate::systems::apply_flashlight_miasma_effects
                .run_if(in_state(SimulationState::Ready))
                .after(crate::systems::diffuse_smoke_field),
        );
        app.add_systems(
            OnExit(UIContextState::InGame),
            crate::systems::reset_miasma_grid,
        );

        metrics::register_all(app);
    }
}

pub struct UnhaunterFogPlugin;

impl Plugin for UnhaunterFogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                crate::systems::spawn_miasma,
                crate::systems::animate_miasma_sprites,
            )
                .run_if(in_state(UIContextState::InGame)),
        );
    }
}
