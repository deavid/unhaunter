use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use unboard_core::entity::GameSprite;
use unboard_core::resources::board_topology::{
    BoardCollisionField, BoardEntityField, BoardTopology,
};
use unclassic_mode_core::components::GCameraArena;
use uncommon_states_core::UIContextState;
use unmission_core::types::SimulationState;

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
    app.add_systems(OnExit(UIContextState::InGame), cleanup_game);
}
