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
    #[derive(Clone)]
    struct InternalWaveEdge {
        position: BoardPosition,
        wave_edge: WaveEdge,
        color: (f32, f32, f32),
        /// Directional source intensity (portion that follows beams)
        dir_src_lux: f32,
        /// Diffuse source intensity (portion that floods rooms)
        diff_src_lux: f32,
    }

    let mut queue = VecDeque::with_capacity(4096);
    let mut propagation_count = 0;
    let directions = [(0, -1, 0), (0, 1, 0), (-1, 0, 0), (1, 0, 0)];

    // IIR factors for turn detection
    const IIR_FACTOR_1: f32 = 0.8;
    const IIR_FACTOR_2: f32 = 0.8;

    // Initial load
    for edge_data in lg.prebaked_wave_edges.iter() {
        if !active_source_ids.contains(&edge_data.source_id) && edge_data.source_id != 0 {
            continue;
        }

        let idx = edge_data.position.ndidx();
        if lfs[idx].lux < edge_data.lux {
            if lfs[idx].lux > 0.0 {
                lfs[idx].color =
                    blend_colors(lfs[idx].color, lfs[idx].lux, edge_data.color, edge_data.lux);
            } else {
                lfs[idx].color = edge_data.color;
            }
            lfs[idx].lux = edge_data.lux;
        }

        // Split initial lighting source power: 95% Highlight, 5% Flood
        queue.push_back(InternalWaveEdge {
            position: edge_data.position.clone(),
            wave_edge: edge_data.wave_edge.clone(),
            color: edge_data.color,
            dir_src_lux: edge_data.wave_edge.src_light_lux * 0.95,
            diff_src_lux: edge_data.wave_edge.src_light_lux * 0.05,
        });
    }

    // BFS
    while let Some(edge_data) = queue.pop_front() {
        let pos = edge_data.position.clone();
        let src_total = edge_data.dir_src_lux + edge_data.diff_src_lux;
        let d = edge_data.wave_edge.distance_travelled;
        let p_lux = src_total / (d * d);

        if p_lux < 0.01 {
            continue;
        }

        for &(dx, dy, dz) in directions.iter() {
            let nx = pos.x + dx;
            let ny = pos.y + dy;
            let nz = pos.z + dz;

            if !is_in_bounds((nx, ny, nz), bf.map_size) {
                continue;
            }

            let n_idx = (nx as usize, ny as usize, nz as usize);

            // Overlap check: skip if already significantly brighter.
            if lfs[n_idx].lux > p_lux * 1.5 {
                continue;
            }

            let collision = &bcf.0[n_idx];

            // Nuanced transparency: air is mostly clear, obstacles are opaque.
            let transparency = if collision.see_through { 0.93 } else { 0.05 };

            let mut next_edge = edge_data.clone();
            let new_pos_f32 = (nx as f32, ny as f32, nz as f32);
            next_edge.wave_edge.current_pos = new_pos_f32;
            next_edge.wave_edge.iir_mean_pos =
                apply_iir_filter(next_edge.wave_edge.iir_mean_pos, new_pos_f32, IIR_FACTOR_1);
            next_edge.wave_edge.iir_mean_iir_mean_pos = apply_iir_filter(
                next_edge.wave_edge.iir_mean_iir_mean_pos,
                next_edge.wave_edge.iir_mean_pos,
                IIR_FACTOR_2,
            );

            // Calculate Turn Penalty (Scattering)
            let (old_dir, recent_dir) = {
                let d1 = (
                    next_edge.wave_edge.iir_mean_pos.0
                        - next_edge.wave_edge.iir_mean_iir_mean_pos.0,
                    next_edge.wave_edge.iir_mean_pos.1
                        - next_edge.wave_edge.iir_mean_iir_mean_pos.1,
                    next_edge.wave_edge.iir_mean_pos.2
                        - next_edge.wave_edge.iir_mean_iir_mean_pos.2,
                );
                let d2 = (
                    next_edge.wave_edge.current_pos.0 - next_edge.wave_edge.iir_mean_pos.0,
                    next_edge.wave_edge.current_pos.1 - next_edge.wave_edge.iir_mean_pos.1,
                    next_edge.wave_edge.current_pos.2 - next_edge.wave_edge.iir_mean_pos.2,
                );
                (d1, d2)
            };

            let dot = {
                let l1 = (old_dir.0.powi(2) + old_dir.1.powi(2) + old_dir.2.powi(2)).sqrt();
                let l2 =
                    (recent_dir.0.powi(2) + recent_dir.1.powi(2) + recent_dir.2.powi(2)).sqrt();
                if l1 > 1e-6 && l2 > 1e-6 {
                    (old_dir.0 * recent_dir.0 + old_dir.1 * recent_dir.1 + old_dir.2 * recent_dir.2)
                        / (l1 * l2)
                } else {
                    1.0
                }
            };
            let dot = (dot + 0.01).clamp(-1.0, 1.0);

            // Scattering conversion: Turns take energy from the "beam" and give it to the "flood".
            let turn_factor = (1.0 - dot).clamp(0.0, 1.0);
            let scatter_amount = next_edge.dir_src_lux * turn_factor * 0.5;

            next_edge.dir_src_lux = (next_edge.dir_src_lux - scatter_amount) * transparency;
            next_edge.diff_src_lux = (next_edge.diff_src_lux + scatter_amount) * transparency;
            next_edge.wave_edge.distance_travelled += 1.0;

            let n_d = next_edge.wave_edge.distance_travelled;
            let n_lux = (next_edge.dir_src_lux + next_edge.diff_src_lux) / (n_d * n_d);

            if n_lux < 0.01 {
                continue;
            }

            // Update Tile
            lfs[n_idx].color =
                blend_colors(lfs[n_idx].color, lfs[n_idx].lux, edge_data.color, n_lux);
            lfs[n_idx].lux += n_lux;

            next_edge.position = BoardPosition {
                x: nx,
                y: ny,
                z: nz,
            };
            queue.push_back(next_edge);
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

        // Calculate a virtual distance to make stair light decay more gracefully.
        // Starting at a larger distance makes the 1/d^2 curve flatter,
        // acting more like an ambient relay than a point source.
        let distance = 8.0;

        // Create a wave edge with the same relative intensity.
        // We multiply stair_lux by distance^2 so that at the entry point,
        // the calculated lux (src / dst) equals the original stair_lux.
        let wave_edge = WaveEdge {
            src_light_lux: stair_lux * (distance * distance),
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
