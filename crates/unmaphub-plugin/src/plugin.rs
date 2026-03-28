use bevy::prelude::*;
use unmaphub_core::states::MapHubState;
use unorchestrator_core::UIContextState;

use crate::difficulty_selection;

pub struct UnhaunterMapHubPlugin;

impl Plugin for UnhaunterMapHubPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MapHubState>();
        app_setup_local(app);
        difficulty_selection::app_setup(app);
    }
}

#[derive(Component, Debug)]
struct MapHubCamera;

pub(crate) fn app_setup_local(app: &mut App) {
    app.add_systems(OnEnter(UIContextState::MapHub), setup_systems)
        .add_systems(OnExit(UIContextState::MapHub), cleanup_systems);
}

fn setup_systems(mut commands: Commands, mut next_state: ResMut<NextState<MapHubState>>) {
    commands.spawn(Camera2d).insert(MapHubCamera);

    next_state.set(MapHubState::DifficultySelection);
}

fn cleanup_systems(mut commands: Commands, q_camera: Query<Entity, With<MapHubCamera>>) {
    // Despawn the camera
    for entity in q_camera.iter() {
        commands.entity(entity).despawn();
    }
}
