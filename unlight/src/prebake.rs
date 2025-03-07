use std::collections::VecDeque;

use crate::utils::{find_wave_edge_tiles, is_in_bounds};
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet, Instant},
};
use ndarray::Array3;
use uncore::{
    behavior::{Behavior, Class},
    components::board::{boardposition::BoardPosition, position::Position},
    resources::board_data::BoardData,
    types::board::prebaked_lighting_data::{LightInfo, PrebakedLightingData, WaveEdge},
};
pub const WAVE_MAX_HISTORY: usize = 12;

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
            let source_visited = visited_by_source
                .entry(source_id)
                .or_insert_with(HashSet::new);
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
            let source_visited = visited_by_source
                .entry(source_id)
                .or_insert_with(HashSet::new);
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

    info!(
        "Prebaked lighting field computed in: {:?}",
        build_start_time.elapsed()
    );
}
