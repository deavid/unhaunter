use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use ndarray::Array3;
use std::collections::VecDeque;
use uncore::{
    behavior::{Behavior, TileState},
    components::board::{boardposition::BoardPosition, position::Position},
    resources::board_data::BoardData,
    types::board::fielddata::LightFieldData,
};

/// Checks if a position is within the board boundaries
pub fn is_in_bounds(pos: (i64, i64, i64), map_size: (usize, usize, usize)) -> bool {
    pos.0 >= 0
        && pos.1 >= 0
        && pos.2 >= 0
        && pos.0 < map_size.0 as i64
        && pos.1 < map_size.1 as i64
        && pos.2 < map_size.2 as i64
}

/// Helper function to check if there are active light sources nearby
pub fn has_active_light_nearby(
    bf: &BoardData,
    active_source_ids: &HashSet<u32>,
    i: usize,
    j: usize,
    k: usize,
) -> bool {
    // Check immediate neighbors plus the current position
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let nx = i as i64 + dx;
                let ny = j as i64 + dy;
                let nz = k as i64 + dz;

                // Skip if out of bounds
                if !is_in_bounds((nx, ny, nz), bf.map_size) {
                    continue;
                }

                let pos = (nx as usize, ny as usize, nz as usize);
                let prebaked_data = &bf.prebaked_lighting[pos];

                if let Some(source_id) = prebaked_data.light_info.source_id {
                    if active_source_ids.contains(&source_id) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Determines if a light is currently active based on its position and behavior
pub fn is_light_active(pos: &BoardPosition, behaviors: &HashMap<BoardPosition, &Behavior>) -> bool {
    if let Some(behavior) = behaviors.get(pos) {
        behavior.p.light.light_emission_enabled
    } else {
        false
    }
}

/// Blend two colors based on their intensity
pub fn blend_colors(
    c1: (f32, f32, f32),
    lux1: f32,
    c2: (f32, f32, f32),
    lux2: f32,
) -> (f32, f32, f32) {
    let total_lux = lux1 + lux2;
    if total_lux <= 0.0 {
        return (1.0, 1.0, 1.0);
    }
    (
        (c1.0 * lux1 + c2.0 * lux2) / total_lux,
        (c1.1 * lux1 + c2.1 * lux2) / total_lux,
        (c1.2 * lux1 + c2.2 * lux2) / total_lux,
    )
}

/// Identifies active light sources in the scene
pub fn identify_active_light_sources(
    bf: &BoardData,
    qt: &Query<(&Position, &Behavior)>,
) -> HashSet<u32> {
    let mut active_source_ids = HashSet::new();

    for (entity, ndidx) in &bf.prebaked_metadata.light_sources {
        let Ok((_pos, behavior)) = qt.get(*entity) else {
            continue;
        };

        if behavior.p.light.light_emission_enabled {
            if let Some(source_id) = bf.prebaked_lighting[*ndidx].light_info.source_id {
                active_source_ids.insert(source_id);
            }
        }
    }
    info!(
        "Active light sources: {}/{} (prebaked) ",
        active_source_ids.len(),
        bf.prebaked_lighting
            .iter()
            .filter(|d| d.light_info.source_id.is_some())
            .count(),
    );

    active_source_ids
}

/// Apply prebaked light contributions from active sources
pub fn apply_prebaked_contributions(
    active_source_ids: &HashSet<u32>,
    bf: &BoardData,
    lfs: &mut Array3<LightFieldData>,
) -> usize {
    let mut tiles_lit = 0;

    // Apply light from active prebaked sources to the lighting field
    for ((i, j, k), prebaked_data) in bf.prebaked_lighting.indexed_iter() {
        let pos_idx = (i, j, k);

        // Get the source ID (if any)
        if let Some(source_id) = prebaked_data.light_info.source_id {
            // Only apply if this source is currently active
            if active_source_ids.contains(&source_id) {
                let lux = prebaked_data.light_info.lux;

                // Skip if no meaningful light contribution
                if lux <= 0.001 {
                    continue;
                }

                // Apply light to this position
                lfs[pos_idx].lux = lux;
                lfs[pos_idx].color = prebaked_data.light_info.color;
                tiles_lit += 1;
            }
        }
    }

    info!("Applied prebaked light: {} tiles lit", tiles_lit);
    tiles_lit
}

/// Update final exposure settings and log statistics
pub fn update_exposure_and_stats(bf: &mut BoardData, lfs: &Array3<LightFieldData>) {
    let tiles_with_light = lfs.iter().filter(|x| x.lux > 0.0).count();
    let total_tiles = bf.map_size.0 * bf.map_size.1 * bf.map_size.2;
    let avg_lux = lfs.iter().map(|x| x.lux).sum::<f32>() / total_tiles as f32;
    let max_lux = lfs.iter().map(|x| x.lux).fold(0.0, f32::max);

    info!(
        "Light field stats: {}/{} tiles lit ({:.2}%), avg: {:.6}, max: {:.6}",
        tiles_with_light,
        total_tiles,
        (tiles_with_light as f32 / total_tiles as f32) * 100.0,
        avg_lux,
        max_lux
    );

    // Calculate exposure
    let total_lux: f32 = lfs.iter().map(|x| x.lux).sum();
    let count = total_tiles as f32;
    let avg_lux = total_lux / count;
    bf.exposure_lux = (avg_lux + 2.0) / 2.0;
    bf.light_field = lfs.clone();

    info!("Final exposure_lux set to: {}", bf.exposure_lux);
}

/// Collects information about door states from entity behaviors
pub fn collect_door_states(
    bf: &BoardData,
    qt: &Query<(&Position, &Behavior)>,
) -> HashMap<(usize, usize, usize), bool> {
    let mut door_states = HashMap::new();
    for entity in &bf.prebaked_metadata.doors {
        if let Ok((pos, behavior)) = qt.get(*entity) {
            let board_pos = pos.to_board_position();
            let idx = board_pos.ndidx();
            let is_open = behavior.state() == TileState::Open;

            // Store the door's open state (true if open, false if closed)
            door_states.insert(idx, is_open);
        }
    }

    info!("Collected {} door states", door_states.len());
    door_states
}

/// Finds wave edge tiles for continuing light propagation
pub fn find_wave_edge_tiles(
    bf: &BoardData,
    active_source_ids: &HashSet<u32>,
    door_states: &HashMap<(usize, usize, usize), bool>,
) -> Vec<(BoardPosition, u32, f32, (f32, f32, f32))> {
    let mut wave_edges = Vec::new();

    // Find all wave edge tiles where light propagation can continue
    for ((i, j, k), prebaked_data) in bf.prebaked_lighting.indexed_iter() {
        // Skip if not a wave edge
        if !prebaked_data.is_wave_edge {
            continue;
        }

        // Skip if no source info
        let source_id = match prebaked_data.light_info.source_id {
            Some(id) => id,
            None => continue,
        };

        // Skip if source is not active
        if !active_source_ids.contains(&source_id) {
            continue;
        }

        // Check if this is adjacent to a door
        let is_near_door = door_states.iter().any(|(&(dx, dy, dz), &_)| {
            (dx as i32 - i as i32).abs() <= 1
                && (dy as i32 - j as i32).abs() <= 1
                && (dz as i32 - k as i32).abs() <= 1
        });

        // If it's near a door, only add if door is open
        if is_near_door {
            let is_near_open_door = door_states.iter().any(|(&(dx, dy, dz), &is_open)| {
                is_open
                    && (dx as i32 - i as i32).abs() <= 1
                    && (dy as i32 - j as i32).abs() <= 1
                    && (dz as i32 - k as i32).abs() <= 1
            });

            if !is_near_open_door {
                continue;
            }
        }

        // Add to wave edges (whether or not it's near a door)
        let pos = BoardPosition {
            x: i as i64,
            y: j as i64,
            z: k as i64,
        };

        wave_edges.push((
            pos,
            source_id,
            prebaked_data.light_info.lux,
            prebaked_data.light_info.color,
        ));
    }

    info!("Found {} wave edge tiles for propagation", wave_edges.len());
    wave_edges
}

/// Propagates light from wave edge tiles past dynamic objects
pub fn propagate_from_wave_edges(
    bf: &BoardData,
    lfs: &mut Array3<LightFieldData>,
    wave_edges: &[(BoardPosition, u32, f32, (f32, f32, f32))],
) -> usize {
    let mut queue = VecDeque::new();
    let mut propagation_count = 0;

    let mut visited: Array3<bool> = Array3::from_elem(bf.map_size, false);

    // Define directions for propagation
    let directions = [(0, 1, 0), (1, 0, 0), (0, -1, 0), (-1, 0, 0)];

    // Add all wave edges to the queue
    for &(ref pos, _, lux, color) in wave_edges {
        queue.push_back((pos.clone(), lux, color));
    }

    // Process queue using BFS
    while let Some((pos, current_lux, color)) = queue.pop_front() {
        // Process each neighbor direction
        for &(dx, dy, dz) in &directions {
            let nx = pos.x + dx;
            let ny = pos.y + dy;
            let nz = pos.z + dz;

            // Skip if out of bounds
            if !is_in_bounds((nx, ny, nz), bf.map_size) {
                continue;
            }

            let neighbor_pos = BoardPosition {
                x: nx,
                y: ny,
                z: nz,
            };
            let neighbor_idx = neighbor_pos.ndidx();

            // Skip if already visited
            if visited[neighbor_idx] {
                continue;
            }

            // Check collision data
            let collision = &bf.collision_field[neighbor_idx];

            if !collision.see_through && !collision.is_dynamic {
                continue;
            }

            // Calculate diminished light (use 0.05 for objects that are opaque but can be changed)
            let falloff = if collision.see_through { 0.65 } else { 0.05 };
            let new_lux = current_lux * falloff;

            // Skip propagating if the contribution is too small
            if lfs[neighbor_idx].lux > new_lux * 2.0 {
                continue;
            }
            // Update light field for neighbor
            if lfs[neighbor_idx].lux > 0.0 {
                lfs[neighbor_idx].color = blend_colors(
                    lfs[neighbor_idx].color,
                    lfs[neighbor_idx].lux,
                    color,
                    new_lux,
                );
            } else {
                lfs[neighbor_idx].color = color;
            }

            lfs[neighbor_idx].lux += new_lux;
            // Add neighbor to queue
            queue.push_back((neighbor_pos, new_lux, color));
            visited[neighbor_idx] = true;

            propagation_count += 1;
        }
    }

    info!("Total light propagations: {}", propagation_count);
    propagation_count
}
