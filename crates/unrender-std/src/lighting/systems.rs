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
use uncore_board::behavior::{Behavior, Class, Orientation};
use uncore_board::resources::board_data::BoardData;
use uncore_board::types::fielddata::LightFieldData;
use uncore_board::types::prebaked_lighting_data::{LightInfo, PrebakedLightingData, WaveEdge};
use unspatial_core::{BoardPosition, Position};

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
    avg_time: &mut Local<(f32, f32)>,
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
        //     stair_wave_edges.len(),
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
    // } else {
    //     info!(
    //         "Light field after stair propagation - Floor 0: {} lit tiles (no additional floors)",
    //         lfs.slice(s![.., .., 0])
    //             .iter()
    //             .filter(|x| x.lux > 0.01)
    //             .count()
    //     );
    // }

    // Apply ambient light to walls
    let time_ambient = Instant::now();
    apply_ambient_light_to_walls(bf, &mut lfs);
    let _ambient_time = time_ambient.elapsed();

    // Calculate exposure and update board data
    update_exposure_and_stats(bf, &lfs);

    let total_time = build_start_time.elapsed().as_secs_f32();
    let tot_cnt = 4.0;
    avg_time.0 = (avg_time.0 * avg_time.1 + total_time * tot_cnt) / (avg_time.1 + tot_cnt);
    avg_time.1 += 1.0;

    // Log detailed performance metrics
    // warn!(
    //     "Lighting field rebuild performance: \
    //     \n  Prebaking: {:?} ({} tiles) \
    //     \n  Main propagation: {:?} ({} propagations) \
    //     \n  Stair preparation: {:?} ({} wave edges) \
    //     \n  Stair propagation: {:?} ({} propagations) \
    //     \n  Ambient light: {:?} \
    //     \n  Total time: {:?} (mean {:.2}ms)",
    //     prebake_time,
    //     initial_tiles_lit,
    //     main_propagation_time,
    //     dynamic_propagation_count,
    //     stair_preparation_time,
    //     stair_wave_edges.len(),
    //     stair_propagation_time,
    //     stair_propagation_count,
    //     ambient_time,
    //     build_start_time.elapsed(),
    //     avg_time.0 * 1000.0
    // );
}

// Applies ambient light to walls based on neighboring lit tiles
fn apply_ambient_light_to_walls(bf: &BoardData, lfs: &mut Array3<LightFieldData>) {
    let _wall_light_start = Instant::now();
    let mut _walls_lit = 0;

    // // Define directions for 4-way connectivity (plus weight)
    let directions = [
        (0, 1, 0, 0.01),
        (1, -1, 0, 0.1),
        (1, 0, 0, 1.0),
        (0, -1, 0, 1.0),
        (-1, 0, 0, 0.01),
    ];

    // Threshold for considering a tile "dark"
    const DARK_THRESHOLD: f32 = 0.1;

    let src_lfs = lfs.clone();

    for ((i, j, k), collision) in bf.collision_field.indexed_iter() {
        // Only process dark tiles
        if src_lfs[(i, j, k)].lux > DARK_THRESHOLD && !collision.is_dynamic {
            continue;
        }
        // Do not process tiles that don't have collision.
        if collision.player_free {
            continue;
        }
        // Collect light from neighbors
        let mut total_lux = 0.0;
        let mut weighted_color_sum = (0.0, 0.0, 0.0);
        let mut weight_sum = 0.0;

        for &(dx, dy, dz, w_factor) in &directions {
            let nx = i as i64 + dx;
            let ny = j as i64 + dy;
            let nz = k as i64 + dz;

            // Skip if out of bounds
            if !is_in_bounds((nx, ny, nz), bf.map_size) {
                continue;
            }

            let n_pos = (nx as usize, ny as usize, nz as usize);
            let neighbor_light = &src_lfs[n_pos];

            // Skip if neighbor has no light
            if neighbor_light.lux <= 0.000000001 {
                continue;
            }

            // Weight based on wall orientation
            let weight = match collision.wall_orientation {
                Orientation::XAxis => {
                    if dy != 0 {
                        2.0
                    } else {
                        1.0
                    }
                }
                Orientation::YAxis => {
                    if dx != 0 {
                        2.0
                    } else {
                        1.0
                    }
                }
                _ => 1.0,
            } * w_factor;

            // Apply ambient factor
            let ambient_factor = 0.3;
            let contribution = neighbor_light.lux * weight * ambient_factor;

            total_lux += contribution;
            weighted_color_sum.0 += neighbor_light.color.0 * weight;
            weighted_color_sum.1 += neighbor_light.color.1 * weight;
            weighted_color_sum.2 += neighbor_light.color.2 * weight;
            weight_sum += weight;
        }

        // Only update if we found lit neighbors
        if weight_sum > 0.0 {
            // Calculate average color
            let avg_color = (
                weighted_color_sum.0 / weight_sum,
                weighted_color_sum.1 / weight_sum,
                weighted_color_sum.2 / weight_sum,
            );

            // Update the light field for this wall
            let lfs_idx = &mut lfs[(i, j, k)];
            lfs_idx.lux = total_lux;
            lfs_idx.color = avg_color;
            _walls_lit += 1;
        }
    }

    // info!(
    //     "Wall ambient light pass: {} walls lit in {:?}",
    //     walls_lit,
    //     wall_light_start.elapsed()
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
                pos,
                source_id,
                data.light_info.lux,
                data.light_info.color,
                2.0, // Distance travelled by light (in tiles, initialized with the light height)
                initial_history,
            ));

            // Mark source position as visited
            let source_visited = visited_by_source.entry(source_id).or_default();
            source_visited.insert((i as i64, j as i64, k as i64));
        }
    }

    // Track statistics
    let mut propagated_tiles = 0;
    let mut wave_edges = 0;

    // Define neighbor directions
    let directions = [
        (0, 1, 0),  // North (+Y)
        (1, 0, 0),  // East (+X)
        (0, -1, 0), // South (-Y)
        (-1, 0, 0), // West (-X)
    ];

    // Process the queue in BFS manner
    while let Some((pos, source_id, src_light_lux, color, distance_travelled, path_history)) =
        propagation_queue.pop_front()
    {
        // Process each neighbor
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

            // Get collision data for the neighbor
            let collision = &bf.collision_field[neighbor_idx];

            // Check if already visited by this source
            let source_visited = visited_by_source.entry(source_id).or_default();
            if source_visited.contains(&(nx, ny, nz)) {
                continue;
            }

            // Check if already visited by another source
            let already_has_different_source =
                prebaked[neighbor_idx].light_info.source_id.is_some()
                    && prebaked[neighbor_idx].light_info.source_id != Some(source_id);

            // Check if this is a dynamic object (e.g., door)
            let is_dynamic_object = collision.is_dynamic;

            // Mark wave edge if:
            // 1. We hit another light source's area, or
            // 2. We hit a dynamic object like a door
            // 3. Transparent things where the player cannot move through, i.e. windows.
            if already_has_different_source || is_dynamic_object || !collision.player_free {
                // Create a trimmed history of the most recent MAX_HISTORY positions
                let mut stored_history = path_history.clone();
                if stored_history.len() > WAVE_MAX_HISTORY {
                    // Keep only the last MAX_HISTORY elements
                    while stored_history.len() > WAVE_MAX_HISTORY {
                        stored_history.pop_front();
                    }
                }
                let pos_last = stored_history.front().unwrap().clone();
                let pos_mid = stored_history
                    .get(stored_history.len() / 2)
                    .unwrap()
                    .clone();
                // Mark the current position as a wave edge with history
                prebaked[pos.ndidx()].wave_edge = Some(WaveEdge {
                    src_light_lux,
                    distance_travelled,
                    current_pos: (pos.x as f32, pos.y as f32, pos.z as f32),
                    iir_mean_pos: (pos_mid.x as f32, pos_mid.y as f32, pos_mid.z as f32),
                    iir_mean_iir_mean_pos: (
                        pos_last.x as f32,
                        pos_last.y as f32,
                        pos_last.z as f32,
                    ),
                });

                wave_edges += 1;

                // If it's the edge, it's because we stopped here. So we stop.
                continue;
            }

            // Check if we can propagate light through this neighbor
            if !collision.see_through {
                continue;
            }

            // Mark this neighbor as visited by this source
            source_visited.insert((nx, ny, nz));

            // Calculate light attenuation with distance
            let new_lux = src_light_lux / (distance_travelled * distance_travelled);

            // Apply the light to this neighbor if it doesn't already have a source
            if prebaked[neighbor_idx].light_info.source_id.is_none() {
                // Set the light properties
                prebaked[neighbor_idx].light_info = LightInfo {
                    source_id: Some(source_id),
                    lux: new_lux,
                    color,
                };

                propagated_tiles += 1;
            }

            // Create updated path history for the neighbor
            let mut new_history = path_history.clone();
            new_history.push_back(neighbor_pos.clone());

            // Continue propagation by adding the neighbor to the queue
            propagation_queue.push_back((
                neighbor_pos,
                source_id,
                src_light_lux,
                color,
                distance_travelled + 1.0,
                new_history,
            ));
        }
    }

    info!(
        "Prebaked light propagation: {} tiles lit, {} wave edges identified",
        propagated_tiles, wave_edges
    );

    // Create a HashSet of all source IDs (during prebaking, all sources are considered active)
    let all_source_ids: HashSet<u32> = visited_by_source.keys().copied().collect();

    // Store the prebaked data in BoardData
    bf.prebaked_lighting = prebaked;
    // Pass the HashSet of all source IDs to find_wave_edge_tiles
    bf.prebaked_wave_edges = find_wave_edge_tiles(bf, &all_source_ids);

    // Add the call to prebake_propagation_data here
    prebake_propagation_data(bf);

    info!(
        "Prebaked lighting field computed in: {:?}",
        build_start_time.elapsed()
    );
}

/// Pre-computes the allowed propagation directions for each light source and tile
fn prebake_propagation_data(bf: &mut BoardData) {
    info!("Computing prebaked propagation directions...");
    let build_start_time = Instant::now();

    let map_size = bf.map_size;

    // Create and initialize the vector of Array2
    bf.prebaked_propagation = vec![
        Array2::from_elem((map_size.0, map_size.1), [false; 4]);
        bf.prebaked_metadata.light_sources.len() + 1
    ];

    // Second pass - compute allowed propagation directions for each light source
    for (source_entity, source_idx) in &bf.prebaked_metadata.light_sources {
        let source_id = match bf.prebaked_metadata.light_source_ids.get(source_entity) {
            Some(id) => *id,
            None => {
                warn!("Light source entity not found in light_source_ids map");
                continue;
            }
        };

        let source_pos = BoardPosition::from_ndidx(*source_idx);

        // Initialize distance field with f32::INFINITY
        let mut distance_field =
            Array3::from_elem((map_size.0, map_size.1, map_size.2), f32::INFINITY);
        distance_field[source_pos.ndidx()] = 0.0;

        let mut queue = VecDeque::new();
        queue.push_front(source_pos.clone()); // Simplified - no need to carry previous position

        while let Some(pos) = queue.pop_back() {
            let p = pos.ndidx();
            let current_distance = distance_field[p];

            // We iterate the 4 neighbors
            for (dir_idx, dir) in [
                (0, -1, 0), // Up (North)
                (0, 1, 0),  // Down (South)
                (-1, 0, 0), // Left (West)
                (1, 0, 0),  // Right (East)
            ]
            .iter()
            .enumerate()
            {
                let neighbor_pos = BoardPosition {
                    x: pos.x + dir.0,
                    y: pos.y + dir.1,
                    z: pos.z + dir.2,
                };

                let n_idx = match neighbor_pos.ndidx_checked(bf.map_size) {
                    Some(idx) => idx,
                    None => continue, // Out of bounds, skip
                };
                let collision = &bf.collision_field[n_idx];

                // Skip if this is a static obstacle (except doors)
                if !collision.see_through && !collision.is_dynamic {
                    continue;
                }

                let new_distance = current_distance + 1.0;

                // Only update if this is a shorter path
                if new_distance < distance_field[n_idx] {
                    distance_field[n_idx] = new_distance;

                    // Mark that we can propagate from pos in direction dir_idx
                    bf.prebaked_propagation[source_id as usize][(pos.x as usize, pos.y as usize)]
                        [dir_idx] = true;

                    queue.push_front(neighbor_pos.clone()); // Simplified - no need for previous position
                }
            }
        }
    }

    info!(
        "Prebaked propagation directions computed in: {:?}",
        build_start_time.elapsed()
    );
}
