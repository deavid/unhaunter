use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use unclassic_mode_core::components::GCameraArena;
use uncommon_states_core::UIContextState;

fn cleanup_game(mut commands: Commands, qc: Query<Entity, (With<GCameraArena>, Without<Remote>)>) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn();
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnExit(UIContextState::InGame), cleanup_game);
}
