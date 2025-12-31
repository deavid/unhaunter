use bevy::prelude::*;
use uncore_board::behavior::Behavior;
use uncore_board::behavior::Orientation;
use uncore_board::resources::board_data::BoardData;
use uncore_board::types::fielddata::CollisionFieldData;
use unspatial_core::Position;

/// Rebuilds the collision data for the board based on the current state of the board and behaviors.
///
/// # Arguments
///
/// * `bf` - A mutable reference to the `BoardData` resource, which stores the collision field.
/// * `qt` - A query for entities with `Position` and `Behavior` components.
pub fn rebuild_collision_data(bf: &mut BoardData, qt: &Query<(Entity, &Position, &Behavior)>) {
    // info!("Collision rebuild");
    assert_eq!(
        bf.collision_field.shape(),
        [bf.map_size.0, bf.map_size.1, bf.map_size.2]
    );
    bf.collision_field.fill(CollisionFieldData::default());

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
        bf.collision_field[bpos.ndidx()] = colfd;
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
        bf.collision_field[bpos.ndidx()] = colfd;
    }
    for (_entity, pos, behavior) in qt
        .iter()
        .filter(|(_e, _p, b)| b.p.movement.stair_offset != 0)
    {
        let bpos = pos.to_board_position();
        bf.collision_field[bpos.ndidx()].stair_offset = behavior.p.movement.stair_offset;
    }
}
