use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use unbehavior::roomdb::RoomDB;
use unboard_core::resources::board_topology::BoardTopology;
use unghost_core::components::GhostBreach;
use unghost_core::components::GhostInfluence;
use unghost_core::resources::haunt_state::HauntState;
use unplayer_core::components::PlayerSprite;
use unrender_std::components::visuals::SpectralInfluence;
use unspatial_core::position::Position;

pub fn assign_ghost_influence(
    commands: &mut Commands,
    movable_objects: &[Entity],
    ghost_spawn_query: &Query<&Position, With<GhostBreach>>,
    player_spawn_query: &Query<&Position, With<PlayerSprite>>,
    position_query: &Query<&Position>,
    roomdb: &RoomDB,
    board_topology: &BoardTopology,
    haunt_state: &HauntState,
) {
    let mut objects_by_floor_with_positions: HashMap<i64, Vec<(Entity, Position)>> = HashMap::new();
    let player_positions: Vec<Position> = player_spawn_query.iter().copied().collect();
    let mut ghost_spawn_points = Vec::new();
    if let Some(ghost_pos) = ghost_spawn_query.iter().next() {
        ghost_spawn_points.push(*ghost_pos);
    } else {
        ghost_spawn_points.push(haunt_state.breach_pos);
    }

    for &entity in movable_objects {
        if let Ok(pos) = position_query.get(entity) {
            let board_pos = pos.to_board_position();
            if roomdb.room_tiles.contains_key(&board_pos) {
                let floor_z = board_pos.z;
                objects_by_floor_with_positions
                    .entry(floor_z)
                    .or_default()
                    .push((entity, *pos));
            }
        }
    }

    if objects_by_floor_with_positions.is_empty() {
        warn!("No movable objects found in valid rooms for influence assignment.");
        return;
    }

    let (_, selected_objects) = crate::selection::select_influence_objects_with_simulation(
        &objects_by_floor_with_positions,
        &ghost_spawn_points,
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
}
