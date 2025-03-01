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
    types::board::prebaked_lighting_data::{
        LightInfo, PendingPropagation, PrebakedLightingData, PropagationDirection,
        SourceContribution,
    },
};

/// Pre-computes static shadow and propagation data for the lighting system.
/// Uses a layer-by-layer BFS approach to ensure waves from different sources
/// propagate at the same rate and appropriately meet in the middle.
pub fn prebake_lighting_field(bf: &mut BoardData, qt: &Query<(&Position, &Behavior)>) {
    info!("Computing prebaked lighting field...");
    let build_start_time = Instant::now();

    // Create a new Array3 with default values
    let mut prebaked = Array3::from_elem(bf.map_size, PrebakedLightingData::default());

    // First pass - identify all light sources and track their properties
    let mut light_source_count = 0;
    let mut next_source_id = 1; // Start source IDs from 1 (0 can be reserved for "no source")

    // IMPORTANT: Set a default transmissivity factor for ALL tiles first
    // This ensures even non-source tiles have proper transmissivity
    for data in prebaked.iter_mut() {
        // Default transmissivity for normal tiles should be 1.0 (not 0.0)
        data.light_info.transmissivity = 1.0;
    }

    // Now process light sources and any specific transmissivity overrides
    for (pos, behavior) in qt.iter() {
        let board_pos = pos.to_board_position();
        let idx = board_pos.ndidx();

        // Set appropriate transmissivity based on the behavior
        prebaked[idx].light_info.transmissivity = behavior.p.light.transmissivity_factor();

        // Check if this entity emits light
        if behavior.p.light.emits_light {
            let lux = behavior.p.light.emmisivity_lumens();
            light_source_count += 1;
            prebaked[idx].light_info = LightInfo {
                is_source: true,
                source_id: next_source_id, // Assign unique ID to this light source
                lux,
                color: behavior.p.light.color(),
                transmissivity: behavior.p.light.transmissivity_factor(),
            };

            // Use the source itself as first contribution
            prebaked[idx].source_contributions.insert(
                next_source_id,
                SourceContribution {
                    lux,
                    color: behavior.p.light.color(),
                    remaining_distance: 30.0, // Max propagation distance
                },
            );

            next_source_id += 1; // Increment for the next source
        }
    }

    info!("Prebaking - Found {} light sources", light_source_count);
    if light_source_count == 0 {
        warn!("No light sources found! Map will be dark.");
    }

    // Track transmissivity statistics
    let transmissivity_deny = prebaked
        .iter()
        .filter(|d| d.light_info.transmissivity <= 0.1)
        .count();
    let transmissivity_allow = prebaked
        .iter()
        .filter(|d| d.light_info.transmissivity >= 0.9)
        .count();
    let total_tiles = bf.map_size.0 * bf.map_size.1 * bf.map_size.2;

    info!(
        "Transmissivity stats: {}/{} tiles are opaque (<=0.1), {}/{} tiles allow light (>=0.9)",
        transmissivity_deny, total_tiles, transmissivity_allow, total_tiles
    );

    // Calculate propagation directions for ALL tiles
    let mut propagation_count = 0;
    for ((i, j, k), data) in prebaked.indexed_iter_mut() {
        let pos = BoardPosition {
            x: i as i64,
            y: j as i64,
            z: k as i64,
        };

        // Improved continuation point detection
        // Not just checking for blocked neighbors, but specifically identifying
        // the interface between light and non-light areas
        let mut has_blocked_neighbor = false;
        let mut has_open_neighbor = false;

        let directions = [
            // North (+Y)
            (0, 1, 0),
            // East (+X)
            (1, 0, 0),
            // South (-Y)
            (0, -1, 0),
            // West (-X)
            (-1, 0, 0),
        ];

        for (dir_i, &(dx, dy, dz)) in directions.iter().enumerate() {
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

            // Check collision data to see if light can pass through
            let collision = &bf.collision_field[neighbor_idx];
            let can_propagate = collision.see_through;

            // Track if this position has both blocked and open neighbors
            if can_propagate {
                has_open_neighbor = true;
            } else {
                has_blocked_neighbor = true;
            }

            // Set propagation data based on direction
            match dir_i {
                0 => data.propagation_dirs.north = can_propagate,
                1 => data.propagation_dirs.east = can_propagate,
                2 => data.propagation_dirs.south = can_propagate,
                3 => data.propagation_dirs.west = can_propagate,
                _ => {}
            }

            if can_propagate {
                propagation_count += 1;
            }
        }

        // Enhanced continuation point marking:
        // 1. Must be at the boundary between open and blocked areas
        // 2. Must be able to propagate light in at least one direction
        // 3. Check if this might be a door based on behavior
        let is_at_boundary = has_blocked_neighbor && has_open_neighbor;
        let can_propagate = data.propagation_dirs.north
            || data.propagation_dirs.east
            || data.propagation_dirs.south
            || data.propagation_dirs.west;

        // Examine both this tile and neighbors for potential door behavior
        let might_be_door_area = {
            let board_pos = BoardPosition {
                x: i as i64,
                y: j as i64,
                z: k as i64,
            };

            let self_collision = &bf.collision_field[board_pos.ndidx()];

            // Check for doorway properties - typically doors have specific collision properties
            // like being impassable but potentially see_through when open
            self_collision.see_through && is_at_boundary
        };

        data.is_continuation_point = (is_at_boundary && can_propagate) || might_be_door_area;
    }

    // Log propagation setup with more details
    let can_propagate_tiles = prebaked
        .iter()
        .filter(|d| {
            d.propagation_dirs.north
                || d.propagation_dirs.east
                || d.propagation_dirs.south
                || d.propagation_dirs.west
        })
        .count();

    let continuation_points = prebaked.iter().filter(|d| d.is_continuation_point).count();

    info!(
        "Propagation: {}/{} tiles can propagate in at least one direction ({:.2}%)",
        can_propagate_tiles,
        total_tiles,
        (can_propagate_tiles as f32 / total_tiles as f32) * 100.0
    );

    info!(
        "Propagation directions set: {} total (avg {:.2} per tile)",
        propagation_count,
        propagation_count as f32 / total_tiles as f32
    );

    info!(
        "Identified {} continuation points for potential dynamic propagation",
        continuation_points
    );

    // Now perform a unified BFS wave propagation to calculate source contributions
    // IMPROVEMENT: Use a layer-by-layer approach to ensure waves propagate at the same rate
    // This makes sure that when waves meet, they are at similar distances from their sources

    // Maps distance -> queue of positions at that distance
    let mut distance_layers: HashMap<
        usize,
        VecDeque<(BoardPosition, u32, f32, f32, (f32, f32, f32))>,
    > = HashMap::new();

    // Maps source_id -> set of visited positions to track wave fronts
    let mut visited_by_source = HashMap::new();

    // Track maximum distance processed
    let max_distance = 30;

    // Initialize distance layers with all light sources at distance 0
    for ((i, j, k), data) in prebaked.indexed_iter() {
        if data.light_info.is_source {
            let pos = BoardPosition {
                x: i as i64,
                y: j as i64,
                z: k as i64,
            };
            let source_id = data.light_info.source_id;
            let lux = data.light_info.lux;
            let color = data.light_info.color;

            // Add to distance layer 0: (position, source_id, remaining_distance, current_lux, color)
            let entry = distance_layers.entry(0).or_default();
            entry.push_back((pos, source_id, max_distance as f32, lux, color));

            // Mark as visited by this source
            let source_visited = visited_by_source
                .entry(source_id)
                .or_insert_with(HashSet::new);
            source_visited.insert((i, j, k));
        }
    }

    // Track propagation statistics
    let mut propagated_light_count = 0;

    // Define directions array once
    let directions = [
        // North (+Y)
        (0, 1, 0, PropagationDirection::North),
        // East (+X)
        (1, 0, 0, PropagationDirection::East),
        // South (-Y)
        (0, -1, 0, PropagationDirection::South),
        // West (-X)
        (-1, 0, 0, PropagationDirection::West),
    ];

    // Process one distance layer at a time
    for distance in 0..=max_distance {
        // Skip if no positions at this distance
        let mut current_layer = match distance_layers.remove(&distance) {
            Some(layer) => layer,
            None => continue,
        };

        info!(
            "Processing distance layer {}/{} with {} positions",
            distance,
            max_distance,
            current_layer.len()
        );

        // Process all positions at the current distance
        while let Some((pos, source_id, remaining_distance, current_lux, color)) =
            current_layer.pop_front()
        {
            // Skip if we've reached the distance limit or light is too dim
            if remaining_distance <= 0.0 || current_lux < 0.001 {
                continue;
            }

            let pos_idx = pos.ndidx();

            // FIX: Store propagation directions in local variables to avoid borrowing issues
            let propagation_dirs = {
                let prebaked_data = &prebaked[pos_idx];
                (
                    prebaked_data.propagation_dirs.north,
                    prebaked_data.propagation_dirs.east,
                    prebaked_data.propagation_dirs.south,
                    prebaked_data.propagation_dirs.west,
                )
            };

            // Propagate in each direction
            for &(dx, dy, dz, dir) in &directions {
                // Check if can propagate in this direction using the cached directions
                let can_propagate = match dir {
                    PropagationDirection::North => propagation_dirs.0,
                    PropagationDirection::East => propagation_dirs.1,
                    PropagationDirection::South => propagation_dirs.2,
                    PropagationDirection::West => propagation_dirs.3,
                };

                if !can_propagate {
                    continue;
                }

                // Calculate neighbor position
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

                // Check if already visited by this source
                let source_visited = visited_by_source
                    .entry(source_id)
                    .or_insert_with(HashSet::new);
                if source_visited.contains(&(nx as usize, ny as usize, nz as usize)) {
                    continue;
                }

                // Mark as visited by this source
                source_visited.insert((nx as usize, ny as usize, nz as usize));

                // Split the immutable and mutable accesses to prebaked
                let (is_continuation_point, transmissivity, see_through) = {
                    let neighbor_prebaked = &prebaked[neighbor_idx];
                    let collision = &bf.collision_field[neighbor_idx];
                    (
                        neighbor_prebaked.is_continuation_point,
                        neighbor_prebaked.light_info.transmissivity,
                        collision.see_through,
                    )
                };

                // If this is a continuation point and it's not see_through, store as pending propagation
                // but DO NOT continue propagation through it during prebaking
                if is_continuation_point && !see_through {
                    prebaked[neighbor_idx]
                        .pending_propagations
                        .push(PendingPropagation {
                            source_id,
                            direction: dir,
                            incoming_lux: current_lux,
                            color,
                            remaining_distance,
                        });
                    continue;
                }

                // Calculate diminished light level with more realistic falloff
                // IMPROVEMENT: Better light falloff model that's more physically accurate
                // Use inverse square law with a minimum to prevent extreme falloff
                // let distance_factor = (distance as f32 + 1.0).powf(1.8).max(1.0);
                // let transmissivity_factor = transmissivity.powf(1.2);
                // let falloff = 0.75 * transmissivity_factor / distance_factor;
                let falloff = 0.75 * transmissivity;

                let new_lux = current_lux * falloff;

                // Skip if light becomes too dim
                if new_lux < 0.000001 {
                    continue;
                }

                // Get the existing contribution or create a new one
                let contribution = {
                    let neighbor_prebaked = &mut prebaked[neighbor_idx];
                    neighbor_prebaked
                        .source_contributions
                        .entry(source_id)
                        .or_insert(SourceContribution {
                            lux: 0.0,
                            color,
                            remaining_distance,
                        })
                };

                // Add the new light contribution
                // IMPROVEMENT: Use max contribution instead of sum when multiple paths exist
                // This prevents additive light effect from the same source via different paths
                let existing_lux = contribution.lux;
                contribution.lux = existing_lux.max(new_lux);

                // Keep the maximum remaining distance
                contribution.remaining_distance = contribution
                    .remaining_distance
                    .max(remaining_distance - 1.0);

                // Add neighbor to the next distance layer
                let next_distance = distance + 1;
                let entry = distance_layers
                    .entry(next_distance)
                    .or_insert_with(VecDeque::new);
                entry.push_back((
                    neighbor_pos,
                    source_id,
                    remaining_distance - 1.0,
                    new_lux,
                    color,
                ));

                propagated_light_count += 1;
            }
        }
    }

    info!(
        "Prebake BFS propagation: {} total light propagations processed",
        propagated_light_count
    );

    // Count how many tiles have contributions
    let tiles_with_contributions = prebaked
        .iter()
        .filter(|x| !x.source_contributions.is_empty())
        .count();

    info!(
        "Source contributions: {}/{} tiles have recorded contributions ({:.2}%)",
        tiles_with_contributions,
        total_tiles,
        (tiles_with_contributions as f32 / total_tiles as f32) * 100.0
    );

    // Store the prebaked data in BoardData
    bf.prebaked_lighting = prebaked;

    info!(
        "Prebaked lighting field computed in: {:?}",
        build_start_time.elapsed()
    );
}
