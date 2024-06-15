mod difficulty_selection;
mod map_selection;

use bevy::prelude::*;

use crate::mainmenu::MCamera;
use crate::root::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum MapHubState {
    MapSelection,
    DifficultySelection,
    #[default]
    None,
}

pub fn app_setup(app: &mut App) {
    app.init_state::<MapHubState>()
        .add_systems(OnEnter(State::MapHub), setup_systems)
        .add_systems(OnExit(State::MapHub), cleanup_systems);

    map_selection::app_setup(app);
    difficulty_selection::app_setup(app);
}

pub fn setup_systems(mut commands: Commands, mut next_state: ResMut<NextState<MapHubState>>) {
    // Create a new camera for the Map Hub UI
    commands.spawn(Camera2dBundle::default()).insert(MCamera);

    // Transition to the map selection screen
    next_state.set(MapHubState::MapSelection);
}

pub fn cleanup_systems(mut commands: Commands, q_camera: Query<Entity, With<MCamera>>) {
    // Despawn the camera
    for entity in q_camera.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
