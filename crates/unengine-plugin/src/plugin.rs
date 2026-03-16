use crate::systems;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use untmxmap_core::resources::maps::Maps;
use untypes_core::states::{AppState, BootState, GameState, SimulationState};

pub struct UnhaunterEngineCorePlugin;

impl Plugin for UnhaunterEngineCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::insert_roles_at_startup);
        app.add_systems(Update, systems::set_boot_ready_when_maps_loaded);
        let is_headless = app
            .world()
            .get_resource::<untypes_core::cli::CliOptions>()
            .map(|cli| cli.dedicated)
            .unwrap_or(false);

        app.add_message::<unevents_core::events::board_topology_rebuild::BoardTopologyToRebuild>();
        app.init_state::<AppState>()
            .init_state::<BootState>()
            .init_state::<GameState>()
            .init_state::<SimulationState>()
            .init_resource::<Maps>()
            .add_loading_state(
                LoadingState::new(AppState::EngineBoot).continue_to_state(AppState::MainMenu),
            );

        if is_headless {
            app.insert_resource(unnoise_core::perlin::PerlinNoise::new_low_mem(1));
        } else {
            app.init_resource::<unnoise_core::perlin::PerlinNoise>();
        }

        crate::boardfield_update::app_setup(app);

        // Runs on all nodes (dedicated + client). Advances SimulationState: Spawning -> Ready.
        app.add_systems(Update, systems::simulation_state_transitions);
    }
}

pub struct UnhaunterEnginePlugin;

impl Plugin for UnhaunterEnginePlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
        crate::pause_ui::app_setup(app);
        crate::hide_mouse::app_setup(app);
    }
}
