use bevy::prelude::*;
use uncore::states::AppState;
use uncore::states::MapHubState;
use uncore::events::map_selected::MapSelectedEvent;

use crate::difficulty_selection;
// map_selection is no longer needed as we now use the unified mission selection

pub struct UnhaunterMapHubPlugin;

impl Plugin for UnhaunterMapHubPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MapHubState>()
            .add_event::<MapSelectedEvent>() // Register the MapSelectedEvent
            .add_systems(OnEnter(AppState::MapHub), setup_systems)
            .add_systems(OnExit(AppState::MapHub), cleanup_systems);
        // We no longer need map_selection::app_setup since we're skipping directly to difficulty selection
        difficulty_selection::app_setup(app);
        // mission_selection is now handled by the unified selector in uncampaign
    }
}

#[derive(Component, Debug)]
struct MapHubCamera;

fn setup_systems(mut commands: Commands, mut next_state: ResMut<NextState<MapHubState>>) {
    // Create a new camera for the Map Hub UI
    commands.spawn(Camera2d).insert(MapHubCamera);

    // Transition directly to difficulty selection, skipping map selection
    next_state.set(MapHubState::DifficultySelection);
}

fn cleanup_systems(mut commands: Commands, q_camera: Query<Entity, With<MapHubCamera>>) {
    // Despawn the camera
    for entity in q_camera.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
