use crate::systems;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unassets_core::resources::maps::Maps;
use untypes_core::states::{AppState, GameState};

pub struct UnhaunterEnginePlugin;

impl Plugin for UnhaunterEnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_state::<GameState>()
            .init_resource::<Maps>()
            .add_loading_state(
                LoadingState::new(AppState::Loading).continue_to_state(AppState::MainMenu),
            );

        app.init_resource::<unnoise_core::perlin::PerlinNoise>();

        systems::app_setup(app);
        crate::pause_ui::app_setup(app);
        crate::hide_mouse::app_setup(app);
        crate::boardfield_update::app_setup(app);
    }
}
