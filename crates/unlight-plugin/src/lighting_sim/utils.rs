use bevy::prelude::*;
use bevy_platform::collections::HashSet;
use ndarray::Array3;
use std::collections::VecDeque;
use unbehavior::behavior::Behavior;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use unlight_core::resources::light_grid::LightGrid;
use unlight_core::types::light::LightFieldData;
use unlight_core::types::prebaked_lighting_data::{WaveEdge, WaveEdgeData};
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::orientation::Orientation;
use unspatial_core::position::Position;

pub const WAVE_MAX_HISTORY: usize = 12;

/// Applies ambient light to walls based on neighboring lit tiles
pub fn apply_ambient_light_to_walls(
    bf: &BoardTopology,
    bcf: &BoardCollisionField,
    lfs: &mut Array3<LightFieldData>,
) {
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

    for ((i, j, k), collision) in bcf.0.indexed_iter() {
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
        }
    }
}

pub fn is_in_bounds(pos: (i64, i64, i64), map_size: (usize, usize, usize)) -> bool {
    pos.0 >= 0
        && pos.1 >= 0
        && pos.2 >= 0
        && pos.0 < map_size.0 as i64
        && pos.1 < map_size.1 as i64
        && pos.2 < map_size.2 as i64
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
    _bf: &BoardTopology,
    lg: &LightGrid,
    qt: &Query<(&Position, &Behavior)>,
) -> HashSet<u32> {
    let mut active_source_ids = HashSet::new();

    // Check if building has power.
    // If no breakers are present, we assume the map has permanent power.
    // If breakers are present, power is ON if at least one breaker is ON.
    let has_power = if lg.prebaked_metadata.breakers.is_empty() {
        true
    } else {
        lg.prebaked_metadata.breakers.iter().any(|&entity| {
            qt.get(entity)
                .map(|(_, behavior)| behavior.state() == unbehavior::state::TileState::On)
                .unwrap_or(false)
        })
    };

    for (entity, ndidx) in &lg.prebaked_metadata.light_sources {
        let Ok((_pos, behavior)) = qt.get(*entity) else {
            continue;
        };

        let can_emit = if behavior.p.is_house_powered {
            behavior.p.light.light_emission_enabled && has_power
        } else {
            behavior.p.light.light_emission_enabled
        };

        if can_emit && let Some(source_id) = lg.prebaked_lighting[*ndidx].light_info.source_id {
            active_source_ids.insert(source_id);
        }
    }
    active_source_ids
}

/// Apply prebaked light contributions from active sources
pub fn apply_prebaked_contributions(
    active_source_ids: &HashSet<u32>,
    _bf: &BoardTopology,
    lg: &LightGrid,
    lfs: &mut Array3<LightFieldData>,
) -> usize {
    let mut tiles_lit = 0;
    let mut v_active = vec![false; lg.prebaked_propagation.len()];
    for source_id in active_source_ids {
        v_active[*source_id as usize] = true;
    }
    // Apply light from active prebaked sources to the lighting field
    for ((i, j, k), prebaked_data) in lg.prebaked_lighting.indexed_iter() {
        let pos_idx = (i, j, k);

        // Get the source ID (if any)
        if let Some(source_id) = prebaked_data.light_info.source_id {
            // Only apply if this source is currently active
            if v_active[source_id as usize] {
                let lux = prebaked_data.light_info.lux;

                // Apply light to this position
                lfs[pos_idx].lux = lux;
                lfs[pos_idx].color = prebaked_data.light_info.color;
                tiles_lit += 1;
            }
        }
    }

    tiles_lit
}

/// Update final exposure settings and log statistics
pub fn update_exposure_and_stats(
    bf: &BoardTopology,
    lg: &mut LightGrid,
    lfs: &Array3<LightFieldData>,
) {
    let total_tiles = bf.map_size.0 * bf.map_size.1 * bf.map_size.2;

    // Calculate exposure
    let total_lux: f32 = lfs.iter().map(|x| x.lux).sum();
    let count = total_tiles as f32;
    let avg_lux = total_lux / count;
    let exposure_lux = (avg_lux + 2.0) / 2.0;

    lg.exposure_lux = exposure_lux;
    lg.light_field = lfs.clone();
}

fn apply_iir_filter(
    current_value: (f32, f32, f32),
    new_value: (f32, f32, f32),
    factor: f32,
) -> (f32, f32, f32) {
    (
        current_value.0 * factor + new_value.0 * (1.0 - factor),
        current_value.1 * factor + new_value.1 * (1.0 - factor),
        current_value.2 * factor + new_value.2 * (1.0 - factor),
    )
}

/// Propagates light from wave edge tiles past dynamic objects
pub fn propagate_from_wave_edges(
    bf: &BoardTopology,
    bcf: &BoardCollisionField,
    lg: &LightGrid,
    lfs: &mut Array3<LightFieldData>,
    active_source_ids: &HashSet<u32>,
) -> usize {
    // New struct to track position history for turn detection
    #[derive(Clone)]
    struct InternalWaveEdge {
        position: BoardPosition,
        wave_edge: WaveEdge,
        source_id: u32,
        color: (f32, f32, f32),
    }

    let mut queue = VecDeque::with_capacity(4096);
    let mut propagation_count = 0;

    // Define directions for propagation
    let directions = [(0, -1, 0), (0, 1, 0), (-1, 0, 0), (1, 0, 0)];

    // IIR factors (adjust these to control the "smoothness")
    const IIR_FACTOR_1: f32 = 0.8; // First level of smoothing
    const IIR_FACTOR_2: f32 = 0.8; // Second level of smoothing

    // Add all wave edges to the queue
    for edge_data in lg.prebaked_wave_edges.iter() {
        if !active_source_ids.contains(&edge_data.source_id) {
            continue;
        }

        queue.push_back(InternalWaveEdge {
            position: edge_data.position.clone(),
            wave_edge: edge_data.wave_edge.clone(),
            source_id: edge_data.source_id,
            color: edge_data.color,
        });
    }

    // Process queue using BFS
    while let Some(edge_data) = queue.pop_front() {
        let pos = edge_data.position;
        let max_lux_possible = edge_data.wave_edge.src_light_lux
            / (edge_data.wave_edge.distance_travelled * edge_data.wave_edge.distance_travelled);

        // If light is too low, skip
        if max_lux_possible < 0.0000001 {
            continue;
        }

        // Special handling for stair wave edges (source_id == 0)
        let is_stair_edge = edge_data.source_id == 0;

        // For stair wave edges, we don't use prebaked propagation directions
        // For regular wave edges, we check the prebaked propagation directions
        let allowed_directions = if is_stair_edge {
            // For stair wave edges, allow all directions
            [true, true, true, true]
        } else {
            // For regular wave edges, use prebaked directions
            match lg
                .prebaked_propagation
                .get(edge_data.source_id as usize)
                .and_then(|arr| arr.get(pos.ndidx()))
            {
                Some(dirs) => *dirs,
                None => {
                    continue;
                }
            }
        };

        // Process each neighbor direction
        for (dir_idx, &(dx, dy, dz)) in directions.iter().enumerate() {
            // Skip if not allowed in this direction
            if !is_stair_edge && !allowed_directions[dir_idx] {
                continue;
            }

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

            // Stop checking early if this neighbor is already too bright
            if lfs[neighbor_idx].lux > max_lux_possible * 4.0 {
                continue;
            }

            // For regular wave edges, skip if neighbor was already in prebaked data
            // For stair wave edges, don't skip
            if !is_stair_edge
                && Some(edge_data.source_id)
                    == lg.prebaked_lighting[neighbor_idx].light_info.source_id
            {
                continue;
            }

            // Check collision data
            let collision = &bcf.0[neighbor_idx];

            // Update wave edge position using IIR filter
            let new_pos_f32 = (nx as f32, ny as f32, nz as f32);

            // Update the current position.
            let mut new_wave_edge = edge_data.wave_edge.clone();
            new_wave_edge.current_pos = new_pos_f32;

            // Apply the first IIR filter to update the mean position.
            new_wave_edge.iir_mean_pos =
                apply_iir_filter(new_wave_edge.iir_mean_pos, new_pos_f32, IIR_FACTOR_1);

            // Apply the second IIR filter to update the mean of the mean position.
            new_wave_edge.iir_mean_iir_mean_pos = apply_iir_filter(
                new_wave_edge.iir_mean_iir_mean_pos,
                new_wave_edge.iir_mean_pos,
                IIR_FACTOR_2,
            );

            let mut turn_penalty = {
                // old_dir is now: from iir_mean_iir_mean_pos to iir_mean_pos
                let old_dir = (
                    new_wave_edge.iir_mean_pos.0 - new_wave_edge.iir_mean_iir_mean_pos.0,
                    new_wave_edge.iir_mean_pos.1 - new_wave_edge.iir_mean_iir_mean_pos.1,
                    new_wave_edge.iir_mean_pos.2 - new_wave_edge.iir_mean_iir_mean_pos.2,
                );

                // recent_dir is now: from iir_mean_pos to current_pos
                let recent_dir = (
                    new_wave_edge.current_pos.0 - new_wave_edge.iir_mean_pos.0,
                    new_wave_edge.current_pos.1 - new_wave_edge.iir_mean_pos.1,
                    new_wave_edge.current_pos.2 - new_wave_edge.iir_mean_pos.2,
                );

                // Normalize vectors and compute dot product
                let old_len =
                    (old_dir.0 * old_dir.0 + old_dir.1 * old_dir.1 + old_dir.2 * old_dir.2).sqrt();
                let recent_len = (recent_dir.0 * recent_dir.0
                    + recent_dir.1 * recent_dir.1
                    + recent_dir.2 * recent_dir.2)
                    .sqrt();

                if old_len > 0.0 && recent_len > 0.0 {
                    let old_norm = (
                        old_dir.0 / old_len,
                        old_dir.1 / old_len,
                        old_dir.2 / old_len,
                    );
                    let recent_norm = (
                        recent_dir.0 / recent_len,
                        recent_dir.1 / recent_len,
                        recent_dir.2 / recent_len,
                    );
                    let dot_product = old_norm.0 * recent_norm.0
                        + old_norm.1 * recent_norm.1
                        + old_norm.2 * recent_norm.2;

                    let dot_product = (dot_product + 0.01).clamp(-1.0, 1.0);
                    const TURN_FACTOR: f32 = 0.6; // Adjust for turn penalty
                    1.0 + (1.0 - dot_product) * TURN_FACTOR
                } else {
                    1.0
                }
            };
            if max_lux_possible > 0.2 {
                turn_penalty += 0.1;
            }

            // Use higher transparency for stair wave edges
            let transparency = if is_stair_edge {
                if collision.see_through {
                    0.9 // Higher transparency for stairs
                } else {
                    0.1 // Still need some penalty for walls
                }
            } else if collision.see_through {
                if collision.player_free && !collision.is_dynamic {
                    0.98 / turn_penalty.min(1.5)
                } else {
                    0.9 / turn_penalty.min(1.5) // Open doors or other see-through dynamic objects
                }
            } else {
                0.05
            };

            let src_light_lux = edge_data.wave_edge.src_light_lux * transparency;
            let distance_travelled = edge_data.wave_edge.distance_travelled;

            // Apply the turn penalty to the light intensity
            let new_lux = src_light_lux / (distance_travelled * distance_travelled);

            new_wave_edge.distance_travelled += 1.0;
            new_wave_edge.src_light_lux = src_light_lux;

            // Skip propagating if the contribution is too small
            if lfs[neighbor_idx].lux > new_lux * 5.0 {
                continue;
            } else if lfs[neighbor_idx].lux > new_lux * 2.0 {
                new_wave_edge.src_light_lux /= 1.1;
            }

            // Update light field for neighbor
            if lfs[neighbor_idx].lux > 0.0 {
                lfs[neighbor_idx].color = blend_colors(
                    lfs[neighbor_idx].color,
                    lfs[neighbor_idx].lux,
                    edge_data.color,
                    new_lux,
                );
            } else {
                lfs[neighbor_idx].color = edge_data.color;
            }

            lfs[neighbor_idx].lux += new_lux;
            if queue.len() > 1_000_000 {
                error_once!("Propagate from waves BFS queue >1M!!");
                continue;
            }
            // Add neighbor to queue with updated history
            queue.push_back(InternalWaveEdge {
                position: neighbor_pos,
                wave_edge: new_wave_edge,
                source_id: edge_data.source_id,
                color: edge_data.color,
            });

            propagation_count += 1;
        }
    }

    propagation_count
}

/// Creates wave edges at stair connections between floors to allow light propagation
pub fn create_stair_wave_edges(
    bf: &BoardTopology,
    bcf: &BoardCollisionField,
    lfs: &Array3<LightFieldData>,
) -> Vec<WaveEdgeData> {
    let mut wave_edges = Vec::new();

    // Process all stair tiles
    for ((i, j, k), collision) in bcf.0.indexed_iter() {
        // Only process stairs
        if collision.stair_offset == 0 {
            continue;
        }

        let pos = (i, j, k);
        let stair_lux = lfs[pos].lux;

        // Skip if no light
        if stair_lux <= 0.0 {
            continue;
        }

        let stair_color = lfs[pos].color;

        // Determine target position based on stair offset
        let target_z = k as i64 + collision.stair_offset as i64;
        if target_z < 0 || target_z >= bf.map_size.2 as i64 {
            continue; // Out of bounds
        }

        let target_pos = (i, j, target_z as usize);
        let target_lux = lfs[target_pos].lux;

        // Only create wave edge if we can bring more light
        if stair_lux <= target_lux {
            continue;
        }

        // Create a wave edge at the target position
        let source_pos = BoardPosition {
            x: i as i64,
            y: j as i64,
            z: k as i64,
        };
        let target_board_pos = BoardPosition {
            x: i as i64,
            y: j as i64,
            z: target_z,
        };

        // Calculate the distance between floors (1.0 for adjacent floors)
        let distance = 1.0;

        // Create a wave edge with the same relative intensity
        let wave_edge = WaveEdge {
            src_light_lux: stair_lux * (distance * distance), // Compensate for distance attenuation
            distance_travelled: distance,
            current_pos: (
                target_board_pos.x as f32,
                target_board_pos.y as f32,
                target_board_pos.z as f32,
            ),
            iir_mean_pos: (
                target_board_pos.x as f32,
                target_board_pos.y as f32,
                target_board_pos.z as f32,
            ),
            iir_mean_iir_mean_pos: (
                source_pos.x as f32,
                source_pos.y as f32,
                source_pos.z as f32,
            ),
        };

        // We don't have a specific source ID for this light, so we'll use a dummy ID
        // that doesn't conflict with existing sources
        let dummy_source_id = 0; // Special ID for stair propagation

        wave_edges.push(WaveEdgeData {
            position: target_board_pos,
            source_id: dummy_source_id,
            lux: stair_lux,
            color: stair_color,
            wave_edge,
        });
    }

    wave_edges
}
