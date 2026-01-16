use bevy::prelude::*;
use unbehavior::Behavior;
use unboard_core::resources::board_topology::BoardTopology;
use unevents_core::events::board_topology_rebuild::BoardTopologyToRebuild;
use unrender_std::utils::collision::rebuild_collision_data;
use unspatial_core::position::Position;

/// Updates the board field based on incoming events and rebuilds collision and lighting data if needed.
///
/// # Arguments
///
/// * `bf` - A mutable reference to the `BoardTopology` resource.
/// * `ev_bdr` - An event reader for `BoardTopologyToRebuild` events.
/// * `qt` - A query for entities with `Position` and `Behavior` components.
fn boardfield_update(
    mut bf: ResMut<BoardTopology>,
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
        rebuild_collision_data(&mut bf, &qt);
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(PostUpdate, boardfield_update)
        .add_message::<BoardTopologyToRebuild>();
}
