use bevy::diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic};
use bevy::prelude::*;
use unboard_core::resources::board_topology::{BoardEntityField, BoardTopology};
use unmetrics_core::metrics::SendMetric;
use unspatial_core::boardposition::MapEntityFieldBPos;
use unspatial_core::position::Position;

const SYNC_MAP_ENTITY_FIELD: DP = DP::const_new("unboard/systems/sync_map_entity_field");

/// Synchronizes the map entity field with the current positions of entities.
///
/// This system updates the `BoardEntityField` resource to reflect the current positions
/// of entities that have moved. It ensures that entities are correctly added to
/// and removed from the map entity field based on their new positions.
fn sync_map_entity_field(
    mut board_entity_field: ResMut<BoardEntityField>,
    board_topology: Res<BoardTopology>,
    mut map_entity_bpos_query: Query<
        (Entity, &Position, &mut MapEntityFieldBPos),
        Changed<Position>,
    >,
) {
    let measure = SYNC_MAP_ENTITY_FIELD.time_measure();
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

    measure.end_ms();
}

/// Populates `BoardEntityField` for entities that are newly given a `MapEntityFieldBPos`.
///
/// `sync_map_entity_field` only reacts to `Changed<Position>`, so a freshly spawned entity
/// that never moves would never enter the grid. This system handles initial insertion by
/// reacting to `Added<MapEntityFieldBPos>`, covering all peers (host, join client, dedicated
/// server) uniformly without any state guard.
fn populate_grid_on_spawn(
    q_added: Query<(Entity, &MapEntityFieldBPos), Added<MapEntityFieldBPos>>,
    mut board_entity_field: ResMut<BoardEntityField>,
    board_topology: Res<BoardTopology>,
) {
    for (entity, bpos) in q_added.iter() {
        if let Some(idx) = bpos.0.ndidx_checked(board_topology.map_size) {
            let cell = &mut board_entity_field.0[idx];
            if !cell.contains(&entity) {
                cell.push(entity);
            }
        }
    }
}

pub(crate) fn register_diagnostics(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(SYNC_MAP_ENTITY_FIELD).with_suffix("ms"));
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            populate_grid_on_spawn,
            sync_map_entity_field.run_if(in_state(uncommon_states_core::UIContextState::InGame)),
        ),
    );
}
