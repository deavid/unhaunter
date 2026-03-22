use bevy::prelude::*;
use unbehavior_core::behavior::Behavior;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use unboard_core::types::fielddata::CollisionFieldData;
use unspatial_core::orientation::Orientation;
use unspatial_core::position::Position;

/// Rebuilds the collision data for the board based on the current state of the board and behaviors.
///
/// # Arguments
///
/// * `bf` - A reference to the `BoardTopology` resource.
/// * `bcf` - A mutable reference to the `BoardCollisionField` resource.
/// * `qt` - A query for entities with `Position` and `Behavior` components.
pub fn rebuild_collision_data(
    bf: &BoardTopology,
    bcf: &mut BoardCollisionField,
    qt: &Query<(Entity, &Position, &Behavior)>,
) {
    assert_eq!(bcf.0.shape(), [bf.map_size.0, bf.map_size.1, bf.map_size.2]);
    bcf.0.fill(CollisionFieldData::default());

    for (_entity, pos, behavior) in qt.iter().filter(|(_e, _p, b)| b.p.movement.walkable) {
        let bpos = pos.to_board_position();
        let colfd = CollisionFieldData {
            player_free: true,
            ghost_free: true,
            see_through: true,
            wall_orientation: Orientation::None,
            is_dynamic: false,
            stair_offset: behavior.p.movement.stair_offset,
        };
        bcf.0[bpos.ndidx()] = colfd;
    }
    for (_entity, pos, behavior) in qt
        .iter()
        .filter(|(_e, _p, b)| b.p.movement.player_collision)
    {
        let bpos = pos.to_board_position();

        let colfd = CollisionFieldData {
            player_free: false,
            ghost_free: !behavior.p.movement.ghost_collision,
            see_through: behavior.p.light.see_through,
            wall_orientation: behavior.orientation(),
            is_dynamic: behavior.p.movement.is_dynamic,
            stair_offset: behavior.p.movement.stair_offset,
        };
        bcf.0[bpos.ndidx()] = colfd;
    }
    for (_entity, pos, behavior) in qt
        .iter()
        .filter(|(_e, _p, b)| b.p.movement.stair_offset != 0)
    {
        let bpos = pos.to_board_position();
        bcf.0[bpos.ndidx()].stair_offset = behavior.p.movement.stair_offset;
    }
}
