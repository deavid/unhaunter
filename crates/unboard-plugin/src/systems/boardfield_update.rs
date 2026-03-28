use bevy::prelude::*;
use unbehavior_core::behavior::Behavior;
use unboard_core::BoardUpdateSet;
use unboard_core::events::board_topology_rebuild::BoardTopologyToRebuild;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use unboard_core::utils::rebuild_collision_data;
use unspatial_core::position::Position;

/// Updates the board field based on incoming events and rebuilds collision and lighting data if needed.
fn boardfield_update(
    bf: Res<BoardTopology>,
    mut bcf: ResMut<BoardCollisionField>,
    mut ev_bdr: MessageReader<BoardTopologyToRebuild>,
    qt: Query<(Entity, &Position, &Behavior)>,
) {
    if ev_bdr.is_empty() {
        return;
    }

    let mut bdr = BoardTopologyToRebuild::default();

    // Merge all the incoming events into a single one.
    for b in ev_bdr.read() {
        if b.collision {
            bdr.collision = true;
        }
        if b.lighting {
            bdr.lighting = true;
        }
    }

    if bdr.collision {
        rebuild_collision_data(&bf, &mut bcf, &qt);
    }
}

pub(crate) fn app_setup(app: &mut App) {
    use unmission_core::types::SimulationState;
    app.add_systems(
        PostUpdate,
        boardfield_update
            .run_if(bevy::prelude::on_message::<BoardTopologyToRebuild>)
            .run_if(not(in_state(SimulationState::Unloaded)))
            .in_set(BoardUpdateSet::Collision),
    );
}
