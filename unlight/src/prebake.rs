use std::collections::VecDeque;

use crate::utils::is_in_bounds;
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet, Instant},
};
use ndarray::Array3;
use uncore::{
    behavior::Behavior,
    components::board::{boardposition::BoardPosition, position::Position},
    resources::board_data::BoardData,
    types::board::prebaked_lighting_data::{LightInfo, PrebakedLightingData},
};

/// Pre-computes static light propagation data using a simplified BFS approach.
///
/// This function:
/// 1. Identifies all light sources
/// 2. Propagates light using BFS
/// 3. Marks wave edges where light stops at dynamic objects or other light sources
pub fn prebake_lighting_field(bf: &mut BoardData, qt: &Query<(&Position, &Behavior)>) {
    info!("Computing prebaked lighting field...");
    let build_start_time = Instant::now();

    // Create a new Array3 with default values
    let mut prebaked = Array3::from_elem(bf.map_size, PrebakedLightingData::default());

    // First pass - identify all light sources and assign unique IDs
    let mut light_source_count = 0;
    let mut next_source_id = 1; // Start from 1, 0 is reserved for "no source"

    // Process all entities to find light sources
    for (pos, behavior) in qt.iter() {
        let board_pos = pos.to_board_position();
        let idx = board_pos.ndidx();

        // Check if this entity emits light
        if behavior.p.light.emits_light {
            let lux = behavior.p.light.emmisivity_lumens();
            let color = behavior.p.light.color();

            light_source_count += 1;
            prebaked[idx].light_info = LightInfo {
                source_id: Some(next_source_id),
                lux,
                color,
            };

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

    // BFS queue for light propagation - (position, source_id, current_lux, color, remaining_distance)
    let mut propagation_queue = VecDeque::new();

    // Initialize queue with all light sources
    for ((i, j, k), data) in prebaked.indexed_iter() {
        if let Some(source_id) = data.light_info.source_id {
            let pos = BoardPosition {
                x: i as i64,
                y: j as i64,
                z: k as i64,
            };

            // Add light source to the queue
            propagation_queue.push_back((
                pos,
                source_id,
                data.light_info.lux,
                data.light_info.color,
                30.0, // Max propagation distance
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
    while let Some((pos, source_id, current_lux, color, remaining_distance)) =
        propagation_queue.pop_front()
    {
        // Skip if light is too dim or distance limit reached
        if remaining_distance <= 0.0 || current_lux < 0.001 {
            continue;
        }

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

            // Check if we can propagate light through this neighbor
            if !collision.see_through {
                continue;
            }

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
            if already_has_different_source || is_dynamic_object {
                // Mark the current position as a wave edge (not the neighbor)
                prebaked[pos.ndidx()].is_wave_edge = true;
                wave_edges += 1;

                // Skip propagation through this neighbor if it's already lit by another source
                if already_has_different_source {
                    continue;
                }
            }

            // Mark this neighbor as visited by this source
            source_visited.insert((nx, ny, nz));

            // Calculate light attenuation with distance
            let falloff = 0.75; // Basic falloff factor
            let new_lux = current_lux * falloff;

            // Skip if light becomes too dim
            if new_lux < 0.001 {
                continue;
            }

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

            // Continue propagation by adding the neighbor to the queue
            propagation_queue.push_back((
                neighbor_pos,
                source_id,
                new_lux,
                color,
                remaining_distance - 1.0,
            ));
        }
    }

    info!(
        "Prebaked light propagation: {} tiles lit, {} wave edges identified",
        propagated_tiles, wave_edges
    );

    // Store the prebaked data in BoardData
    bf.prebaked_lighting = prebaked;

    info!(
        "Prebaked lighting field computed in: {:?}",
        build_start_time.elapsed()
    );
}
