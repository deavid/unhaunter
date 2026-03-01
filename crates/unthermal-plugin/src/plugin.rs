use crate::metrics;
use crate::systems::*;
use bevy::prelude::*;
use untypes_core::states::{AppState, SimulationState};

pub struct UnhaunterThermalPlugin;

impl Plugin for UnhaunterThermalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            temperature_update.run_if(in_state(SimulationState::Ready)),
        )
        .add_systems(
            Update,
            (
                init_thermal_grid_allocation,
                init_thermal_grid_content.after(init_thermal_grid_allocation),
            ),
        )
        .add_systems(OnExit(AppState::InGame), reset_thermal_grid);

        metrics::register_all(app);
    }
}
