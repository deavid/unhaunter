use bevy::prelude::*;
use unboard_core::BoardUpdateSet;
use uncommon_states_core::UIContextState;
use unmission_core::types::SimulationState;

use unlight_core::flashlight::ActiveFlashlights;
use unlight_core::sets::LightUpdateSet;

use crate::{lighting_sim, maplight, metrics};

pub struct UnhaunterLightCorePlugin;

impl Plugin for UnhaunterLightCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveFlashlights>()
            .configure_sets(
                PostUpdate,
                LightUpdateSet::Gather
                    .after(BoardUpdateSet::Lighting)
                    .run_if(in_state(UIContextState::InGame).and(in_state(SimulationState::Ready))),
            )
            .add_systems(PreUpdate, lighting_sim::systems::init_light_grid)
            .add_systems(
                Update,
                lighting_sim::systems::prebake_lighting_on_level_ready,
            )
            .add_systems(
                PostUpdate,
                (
                    lighting_sim::systems::rebuild_lighting_field
                        .in_set(BoardUpdateSet::Lighting)
                        .after(BoardUpdateSet::Collision)
                        .run_if(not(in_state(SimulationState::Unloaded))),
                    maplight::systems::gathering::player_visibility_system
                        .after(BoardUpdateSet::Lighting)
                        .run_if(in_state(SimulationState::Ready)),
                    (
                        maplight::systems::gathering::gather_flashlights_system,
                        maplight::systems::gathering::update_exposure_system,
                    )
                        .chain()
                        .in_set(LightUpdateSet::Gather),
                ),
            )
            .add_systems(
                OnExit(UIContextState::InGame),
                lighting_sim::systems::reset_light_grid,
            );
        metrics::register_all(app);
    }
}
