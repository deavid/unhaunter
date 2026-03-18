use bevy::prelude::*;
use unboard_core::BoardUpdateSet;
use untypes_core::states::{AppState, SimulationState};

use unlight_core::resources::light_grid::LightGrid;

use crate::{lighting_sim, maplight, metrics, systems::power_visuals};

pub struct UnhaunterLightCorePlugin;

impl Plugin for UnhaunterLightCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightGrid>()
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
                ),
            )
            .add_systems(
                OnExit(AppState::InGame),
                lighting_sim::systems::reset_light_grid,
            );
        metrics::register_all(app);
    }
}

pub struct UnhaunterLightPlugin;

impl Plugin for UnhaunterLightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                power_visuals::update_power_visuals,
                maplight::systems::gathering::gather_flashlights_system,
                maplight::systems::gathering::update_exposure_system,
                maplight::systems::tiles::apply_lighting_to_tiles_system,
                maplight::systems::sprites::apply_lighting_to_sprites_system,
                maplight::systems::sprites::highlight_placement_tiles_system,
            )
                .chain()
                .after(BoardUpdateSet::Lighting)
                .run_if(in_state(AppState::InGame).and(in_state(SimulationState::Ready))),
        );
        maplight::systems::app_setup(app);
    }
}
