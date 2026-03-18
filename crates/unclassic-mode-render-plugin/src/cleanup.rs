use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use unboard_core::resources::board_topology::{
    BoardCollisionField, BoardEntityField, BoardTopology,
};
use unrender_std::components::game::GameSprite;
use untags_core::game::GCameraArena;
use untypes_core::states::{AppState, SimulationState};

fn cleanup_game(
    mut commands: Commands,
    qc: Query<Entity, (With<GCameraArena>, Without<Remote>)>,
    qgs: Query<Entity, (With<GameSprite>, Without<Remote>)>,
    mut bf: ResMut<BoardTopology>,
    mut bcf: ResMut<BoardCollisionField>,
    mut bef: ResMut<BoardEntityField>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
) {
    bf.reset();
    bcf.reset();
    bef.reset();
    next_sim_state.set(SimulationState::Unloaded);

    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn();
    }

    // Despawn game sprites if not used
    for gs in qgs.iter() {
        commands.entity(gs).despawn();
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnExit(AppState::InGame), cleanup_game);
}
