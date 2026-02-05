use bevy::prelude::*;
use unboard_core::resources::board_topology::{BoardEntityField, BoardTopology};
use unspatial_core::boardposition::MapEntityFieldBPos;
use unspatial_core::position::Position;

/// Synchronizes the map entity field with the current positions of entities.
///
/// This system updates the `BoardEntityField` resource to reflect the current positions
/// of entities that have moved. It ensures that entities are correctly added to
/// and removed from the map entity field based on their new positions.
///
/// Optimized to only process entities within a reasonable radius of the player
/// using the BoardEntityField for efficient entity lookup.
fn sync_map_entity_field(
    mut board_entity_field: ResMut<BoardEntityField>,
    board_topology: Res<BoardTopology>,
    mut map_entity_bpos_query: Query<
        (Entity, &Position, &mut MapEntityFieldBPos),
        Changed<Position>,
    >,
) {
    let map_size = board_topology.map_size;
    let mut to_update = Vec::new();

    for (entity, pos, mut old_bpos) in map_entity_bpos_query.iter_mut() {
        let current_bpos = pos.to_board_position_size(map_size);
        if old_bpos.0 == current_bpos {
            continue;
        }

        to_update.push((entity, current_bpos.clone(), old_bpos.0.clone()));
        old_bpos.0 = current_bpos;
    }

    for (entity, current_bpos, old_bpos) in to_update {
        if let Some(entity_vec) = board_entity_field.0.get_mut(old_bpos.ndidx()) {
            entity_vec.retain(|&e| e != entity);
        }

        if let Some(entity_vec) = board_entity_field.0.get_mut(current_bpos.ndidx())
            && !entity_vec.contains(&entity)
        {
            entity_vec.push(entity);
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, sync_map_entity_field);
}
