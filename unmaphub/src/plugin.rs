use bevy::prelude::*;
use uncore::states::AppState;
use uncore::states::MapHubState;

use crate::difficulty_selection;
use crate::map_selection;

pub struct UnhaunterMapHubPlugin;

impl Plugin for UnhaunterMapHubPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MapHubState>()
            .add_systems(OnEnter(AppState::MapHub), setup_systems)
            .add_systems(OnExit(AppState::MapHub), cleanup_systems);
        map_selection::app_setup(app);
        difficulty_selection::app_setup(app);
    }
}

#[derive(Component, Debug)]
struct MapHubCamera;

fn setup_systems(mut commands: Commands, mut next_state: ResMut<NextState<MapHubState>>) {
    // Create a new camera for the Map Hub UI
    commands.spawn(Camera2d).insert(MapHubCamera);

    // Transition to the map selection screen
    next_state.set(MapHubState::MapSelection);
}

fn cleanup_systems(mut commands: Commands, q_camera: Query<Entity, With<MapHubCamera>>) {
    // Despawn the camera
    for entity in q_camera.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
