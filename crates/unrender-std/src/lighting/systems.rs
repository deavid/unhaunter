use crate::lighting::utils::{
    WAVE_MAX_HISTORY, apply_prebaked_contributions, create_stair_wave_edges, find_wave_edge_tiles,
    identify_active_light_sources, is_in_bounds, propagate_from_wave_edges,
    update_exposure_and_stats,
};
use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use bevy_platform::collections::HashSet;
use bevy_platform::time::Instant;
use ndarray::{Array2, Array3};
use std::collections::VecDeque;
use unboard_core::behavior::{Behavior, Class};
use unboard_core::resources::board_data::BoardData;
use unboard_core::types::fielddata::LightFieldData;
use unboard_core::types::prebaked_lighting_data::{LightInfo, PrebakedLightingData, WaveEdge};
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;

/// Rebuilds the lighting field based on the current state of the board and behaviors
/// by switching between legacy and new implementations.
///
/// This function iterates through all entities with `Position` and `Behavior` components,
/// calculates the light emitted and transmitted by each entity, and then propagates
/// the light throughout the board using a multi-step process.
///
/// # Arguments
///
/// * `bf` - A mutable reference to the `BoardData` resource, which stores the lighting field.
/// * `qt` - A query for entities with `Position` and `Behavior` components.
pub fn rebuild_lighting_field(
    bf: &mut BoardData,
    qt: &Query<(&Position, &Behavior)>,
    _avg_time: &mut Local<(f32, f32)>,
) {
    // info!("Starting rebuild_lighting_field using prebaked data");
    let build_start_time = Instant::now();

    // Create a new light field with default values
    let mut lfs = Array3::from_elem(bf.map_size, LightFieldData::default());

    // Identify active light sources
    let active_source_ids = identify_active_light_sources(bf, qt);

    // Apply prebaked contributions from active sources
    let _initial_tiles_lit = apply_prebaked_contributions(&active_source_ids, bf, &mut lfs);
    let _prebake_time = build_start_time.elapsed();

    // First pass of light propagation from wave edges
    let time_main_propagation = Instant::now();
    let _dynamic_propagation_count = propagate_from_wave_edges(bf, &mut lfs, &active_source_ids);
    let _main_propagation_time = time_main_propagation.elapsed();

    // Log light statistics before stair propagation
    // if bf.map_size.2 > 1 {
    //     info!(
    //         "Light field before stair propagation - Floor 0: {} lit tiles, Floor 1: {} lit tiles",
    //         lfs.slice(s![.., .., 0])
    //             .iter()
    //             .filter(|x| x.lux > 0.01)
    //             .count(),
    //         lfs.slice(s![.., .., 1])
    //             .iter()
    //             .filter(|x| x.lux > 0.01)
    //             .count()
    //     );
    // } else {
    //     info!(
    //         "Light field before stair propagation - Floor 0: {} lit tiles (no additional floors)",
    //         lfs.slice(s![.., .., 0])
    //             .iter()
    //             .filter(|x| x.lux > 0.01)
    //             .count()
    //     );
    // }

    // Create wave edges from stairs and add them to a temporary list
    let time_stair_preparation = Instant::now();
    let stair_wave_edges = create_stair_wave_edges(bf, &lfs);
    let _stair_preparation_time = time_stair_preparation.elapsed();

    // If we found stair wave edges, do a second pass of propagation using those
    let mut _stair_propagation_count = 0;
    let time_stair_propagation = Instant::now();
    if !stair_wave_edges.is_empty() {
        // Save original wave edges
        let original_wave_edges = bf.prebaked_wave_edges.clone();

        // Temporarily replace wave edges with stair wave edges
        bf.prebaked_wave_edges = stair_wave_edges.clone();

        // Propagate from the stair wave edges (using all source IDs to ensure our dummy ID is included)
        let all_sources: HashSet<u32> =
            (0..=active_source_ids.iter().max().unwrap_or(&0) + 1).collect();
        // info!(
        //     "Starting stair light propagation with {} wave edges",
        //     stair_wave_edges.len()
        // );
        _stair_propagation_count = propagate_from_wave_edges(bf, &mut lfs, &all_sources);

        // Restore original wave edges
        bf.prebaked_wave_edges = original_wave_edges;
    }
    let _stair_propagation_time = time_stair_propagation.elapsed();

    // Log light statistics after stair propagation
    // if bf.map_size.2 > 1 {
    //     info!(
    //         "Light field after stair propagation - Floor 0: {} lit tiles, Floor 1: {} lit tiles",
    //         lfs.slice(s![.., .., 0])
    //             .iter()
    //             .filter(|x| x.lux > 0.01)
    //             .count(),
    //         lfs.slice(s![.., .., 1])
    //             .iter()
    //             .filter(|x| x.lux > 0.01)
    //             .count()
    //     );
    // }

    // Update exposure and stats
    update_exposure_and_stats(bf, &lfs);

    let _total_time = build_start_time.elapsed();
    // info!(
    //     "Rebuild lighting field: total={:?}, prebake={:?}, main_prop={:?}, stair_prep={:?}, stair_prop={:?}",
    //     total_time,
    //     prebake_time,
    //     main_propagation_time,
    //     stair_preparation_time,
    //     stair_propagation_time
    // );
}

/// Pre-computes static light propagation data using a simplified BFS approach.
///
/// This function:
/// 1. Identifies all light sources
/// 2. Propagates light using BFS
/// 3. Marks wave edges where light stops at dynamic objects or other light sources
pub fn prebake_lighting_field(bf: &mut BoardData, qt: &Query<(Entity, &Position, &Behavior)>) {
    info!("Computing prebaked lighting field...");
    let build_start_time = Instant::now();

    // Create a new Array3 with default values
    let mut prebaked = Array3::from_elem(bf.map_size, PrebakedLightingData::default());

    // First pass - identify all light sources and assign unique IDs
    let mut light_source_count = 0;
    let mut next_source_id = 1; // Start from 1, 0 is reserved for "no source"
    bf.prebaked_metadata.light_source_ids.clear();

    // First pass - identify all light sources and assign sequential IDs

    bf.prebaked_metadata = Default::default();
    // Process all entities to find light sources
    for (entity, pos, behavior) in qt.iter() {
        let board_pos = pos.to_board_position();
        let idx = board_pos.ndidx();
        let is_door = behavior.key_cvo().class == Class::Door;

        if is_door {
            bf.prebaked_metadata.doors.push(entity);
        }

        // Check if this entity emits light
        if behavior.p.light.can_emit_light {
            let lux = behavior.p.light.emission_power.exp();
            let color = behavior.p.light.color();

            light_source_count += 1;
            prebaked[idx].light_info = LightInfo {
                source_id: Some(next_source_id),
                lux,
                color,
            };
            bf.prebaked_metadata
                .light_source_ids
                .insert(entity, next_source_id);
            bf.prebaked_metadata.light_sources.push((entity, idx));
            next_source_id += 1;
        }
    }

    info!("Prebaking - Found {} light sources", light_source_count);
    if light_source_count == 0 {
        warn!("No light sources found! Map will be dark.");
        return;
    }

    // Track positions visited by each light source to prevent overlap
    let mut visited_by_source: HashMap<u32, HashSet<(i64, i64, i64)>> = HashMap::new();

    // BFS queue for light propagation - (position, source_id, current_lux, color, remaining_distance, path_history)
    let mut propagation_queue = VecDeque::new();

    // Initialize queue with all light sources
    for ((i, j, k), data) in prebaked.indexed_iter() {
        if let Some(source_id) = data.light_info.source_id {
            let pos = BoardPosition {
                x: i as i64,
                y: j as i64,
                z: k as i64,
            };

            // Create initial history with just the source position
            let mut initial_history = VecDeque::new();
            initial_history.push_back(pos.clone());

            // Add light source to the queue
            propagation_queue.push_back((
                pos.clone(),
                source_id,
                data.light_info.lux,
                data.light_info.color,
                2.0, // Distance travelled by light (in tiles, initialized with the light height)
                initial_history,
            ));

            // Mark source position as visited
            visited_by_source
                .entry(source_id)
                .or_default()
                .insert((pos.x, pos.y, pos.z));
        }
    }

    // Initialize prebaked propagation directions
    // This stores which directions light flows into each cell for each source
    // Format: source_id -> (x, y) -> [bool; 4] (N, S, W, E)
    let mut propagation_directions: Vec<Array2<[bool; 4]>> =
        vec![
            Array2::from_elem((bf.map_size.0, bf.map_size.1), [false; 4]);
            next_source_id as usize
        ];

    // Process BFS queue
    let mut tiles_processed = 0;
    let directions = [(0, -1, 0), (0, 1, 0), (-1, 0, 0), (1, 0, 0)]; // N, S, W, E

    while let Some((pos, source_id, current_lux, color, distance_travelled, history)) =
        propagation_queue.pop_front()
    {
        tiles_processed += 1;

        // Stop if light is too dim
        if current_lux < 0.001 {
            continue;
        }

        // Check neighbors
        for (dir_idx, (dx, dy, dz)) in directions.iter().enumerate() {
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

            // Skip if already visited by this source
            if visited_by_source
                .entry(source_id)
                .or_default()
                .contains(&(nx, ny, nz))
            {
                continue;
            }

            // Check collision
            let collision = &bf.collision_field[neighbor_idx];

            // If it's a wall/obstacle, stop propagation but mark as visited
            if !collision.see_through {
                visited_by_source
                    .entry(source_id)
                    .or_default()
                    .insert((nx, ny, nz));
                continue;
            }

            // Calculate new lux based on distance
            let new_distance = distance_travelled + 1.0;
            let src_lux = prebaked[pos.ndidx()].light_info.lux
                * (distance_travelled * distance_travelled)
                / (new_distance * new_distance);

            // If this is a dynamic object (door, etc), create a wave edge and stop
            if collision.is_dynamic {
                // Calculate mean position from history (last N steps)
                let history_len = history.len();
                let steps_to_avg = history_len.min(WAVE_MAX_HISTORY);
                let mut sum_x = 0.0;
                let mut sum_y = 0.0;
                let mut sum_z = 0.0;

                for i in 0..steps_to_avg {
                    let p = &history[history_len - 1 - i];
                    sum_x += p.x as f32;
                    sum_y += p.y as f32;
                    sum_z += p.z as f32;
                }

                let mean_pos = (
                    sum_x / steps_to_avg as f32,
                    sum_y / steps_to_avg as f32,
                    sum_z / steps_to_avg as f32,
                );

                // Calculate mean of mean (smoother)
                // For simplicity in prebake, we'll just use the mean again or a weighted version
                // But to match the runtime structure, we'll just use the mean
                let mean_mean_pos = mean_pos;

                // Store wave edge data
                prebaked[neighbor_idx].wave_edge = Some(WaveEdge {
                    src_light_lux: src_lux,
                    distance_travelled: new_distance,
                    current_pos: (nx as f32, ny as f32, nz as f32),
                    iir_mean_pos: mean_pos,
                    iir_mean_iir_mean_pos: mean_mean_pos,
                });

                // Also store light info so we know which source this belongs to
                prebaked[neighbor_idx].light_info = LightInfo {
                    source_id: Some(source_id),
                    lux: src_lux,
                    color,
                };

                // Mark as visited so we don't process again for this source
                visited_by_source
                    .entry(source_id)
                    .or_default()
                    .insert((nx, ny, nz));

                // Record propagation direction
                // Mark that light flows FROM pos TO neighbor_pos
                // This means neighbor_pos receives light from the direction opposite to (dx, dy)
                // But our array stores "allowed directions", so we mark the direction we came from as allowed?
                // Actually, let's look at how it's used:
                // In runtime: `allowed_directions[dir_idx]` checks if we can go in `directions[dir_idx]`
                // So we should mark the direction we are GOING as allowed for the CURRENT cell
                // Wait, the logic in runtime is:
                // `bf.prebaked_propagation.get(source_id).get(pos).dirs`
                // So at `pos`, we store which directions are valid to exit.
                if let Some(prop_grid) = propagation_directions.get_mut(source_id as usize) {
                    if let Some(dirs) = prop_grid.get_mut((pos.x as usize, pos.y as usize)) {
                        dirs[dir_idx] = true;
                    }
                }

                continue;
            }

            // Normal propagation
            // Update light info at neighbor
            // If multiple sources reach here, we might want to blend or keep the brightest
            // For prebaking, we'll just overwrite if we're the first one (checked by visited)
            // or maybe we should accumulate?
            // The current structure assumes one source per cell in prebaked data for simplicity
            // But in reality, multiple sources can overlap.
            // The `visited_by_source` ensures we don't process the same cell twice for the SAME source.
            // But different sources can visit the same cell.
            // However, `prebaked[neighbor_idx]` can only store ONE source ID.
            // This is a limitation of the current data structure.
            // We'll stick to "first come first served" or "brightest wins" logic?
            // Let's use "brightest wins" for the stored ID, but we still propagate.

            let existing_lux = prebaked[neighbor_idx].light_info.lux;
            if src_lux > existing_lux {
                prebaked[neighbor_idx].light_info = LightInfo {
                    source_id: Some(source_id),
                    lux: src_lux,
                    color,
                };
            }

            // Mark as visited
            visited_by_source
                .entry(source_id)
                .or_default()
                .insert((nx, ny, nz));

            // Record propagation direction
            if let Some(prop_grid) = propagation_directions.get_mut(source_id as usize) {
                if let Some(dirs) = prop_grid.get_mut((pos.x as usize, pos.y as usize)) {
                    dirs[dir_idx] = true;
                }
            }

            // Update history
            let mut new_history = history.clone();
            new_history.push_back(neighbor_pos.clone());
            if new_history.len() > WAVE_MAX_HISTORY {
                new_history.pop_front();
            }

            // Add to queue
            propagation_queue.push_back((
                neighbor_pos,
                source_id,
                src_lux,
                color,
                new_distance,
                new_history,
            ));
        }
    }

    // Store the results in BoardData
    bf.prebaked_lighting = prebaked;
    bf.prebaked_propagation = propagation_directions;

    // Extract wave edges for runtime use
    let active_source_ids: HashSet<u32> = (1..next_source_id).collect();
    bf.prebaked_wave_edges = find_wave_edge_tiles(bf, &active_source_ids);

    let total_time = build_start_time.elapsed();
    info!(
        "Prebaked lighting complete in {:?}. Processed {} tiles. Found {} wave edges.",
        total_time,
        tiles_processed,
        bf.prebaked_wave_edges.len()
    );
}
