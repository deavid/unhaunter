use super::{level, roomchanged, systems, ui};
use bevy::prelude::*;
use uncore::components::game_config::GameConfig;
use uncore::states::AppState;

pub struct UnhaunterGamePlugin;

impl Plugin for UnhaunterGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .add_systems(OnEnter(AppState::InGame), systems::setup)
            .add_systems(OnExit(AppState::InGame), systems::cleanup)
            .add_systems(Update, systems::keyboard);
        level::app_setup(app);
        ui::app_setup(app);
        roomchanged::app_setup(app);
    }
}
