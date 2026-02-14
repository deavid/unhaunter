use crate::metrics;
use crate::systems::*;
use bevy::prelude::*;
use unthermal_core::resources::ThermalGrid;

pub struct UnhaunterThermalPlugin;

impl Plugin for UnhaunterThermalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ThermalGrid>()
            .add_systems(Update, temperature_update)
            .add_systems(
                Update,
                (
                    init_thermal_grid_allocation,
                    init_thermal_grid_content.after(init_thermal_grid_allocation),
                ),
            );

        metrics::register_all(app);
    }
}
