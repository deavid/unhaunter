use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use unboard_core::components::spawning::PlayerSpawnPoint;
use unboard_core::resources::board_topology::BoardTopology;
use unboard_core::resources::roomdb::RoomTopology;
use unghost_core::components::logic::ghost_influence::GhostInfluence;
use unlight_core::spectral::SpectralInfluence;
use unspatial_core::position::Position;

pub(crate) fn assign_ghost_influence(
    commands: &mut Commands,
    movable_objects: &[Entity],
    ghost_spawn_points: &[Position],
    player_spawn_query: &Query<&Position, With<PlayerSpawnPoint>>,
    position_query: &Query<&Position>,
    room_topology: &RoomTopology,
    board_topology: &BoardTopology,
) -> Position {
    let mut objects_by_floor_with_positions: HashMap<i64, Vec<(Entity, Position)>> = HashMap::new();
    let player_positions: Vec<Position> = player_spawn_query.iter().copied().collect();
    let mut missing_position_count = 0usize;
    let mut non_integer_z_count = 0usize;
    let mut outside_room_count = 0usize;
    let mut accepted_count = 0usize;
    let mut sample_non_integer_z = Vec::new();
    let mut sample_outside_room = Vec::new();

    for &entity in movable_objects {
        let Ok(pos) = position_query.get(entity) else {
            missing_position_count += 1;
            continue;
        };

        let board_pos = pos.to_board_position();

        if (pos.z - pos.z.round()).abs() > 0.05 {
            non_integer_z_count += 1;
            if sample_non_integer_z.len() < 5 {
                sample_non_integer_z.push(format!(
                    "entity={entity:?} pos=({:.2}, {:.2}, {:.2}) board_pos={board_pos:?}",
                    pos.x, pos.y, pos.z
                ));
            }
            continue;
        }

        if !room_topology.room_tiles.contains_key(&board_pos) {
            outside_room_count += 1;
            if sample_outside_room.len() < 5 {
                sample_outside_room.push(format!(
                    "entity={entity:?} pos=({:.2}, {:.2}, {:.2}) board_pos={board_pos:?}",
                    pos.x, pos.y, pos.z
                ));
            }
            continue;
        }

        accepted_count += 1;
        let floor_z = board_pos.z;
        objects_by_floor_with_positions
            .entry(floor_z)
            .or_default()
            .push((entity, *pos));
    }

    if objects_by_floor_with_positions.is_empty() {
        error!(
            total_movable = movable_objects.len(),
            missing_position_count,
            non_integer_z_count,
            outside_room_count,
            accepted_count,
            sample_non_integer_z = ?sample_non_integer_z,
            sample_outside_room = ?sample_outside_room,
            "No movable objects found in valid rooms for influence assignment."
        );
        return ghost_spawn_points
            .first()
            .copied()
            .unwrap_or(Position::new_i64(0, 0, 0));
    }

    debug!(
        total_movable = movable_objects.len(),
        missing_position_count,
        non_integer_z_count,
        outside_room_count,
        accepted_count,
        floors = objects_by_floor_with_positions.len(),
        "Haunted object influence candidate filtering summary"
    );

    let (selected_spawn_point, selected_objects) =
        crate::selection::select_influence_objects_with_simulation(
            &objects_by_floor_with_positions,
            ghost_spawn_points,
            &player_positions,
            board_topology,
        );

    for (entity, influence_type) in selected_objects {
        commands.entity(entity).insert((
            GhostInfluence {
                influence_type,
                charge_value: 0.0,
            },
            SpectralInfluence::default(),
        ));
    }

    selected_spawn_point
}
