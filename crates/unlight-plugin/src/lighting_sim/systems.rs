use super::utils::*;
use bevy::prelude::*;
use bevy_platform::collections::{HashMap, HashSet};
use bevy_platform::time::Instant;
use ndarray::Array3;
use std::collections::VecDeque;
use unbehavior_core::behavior::Behavior;
use unboard_core::events::board_topology_rebuild::BoardTopologyToRebuild;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use unlight_core::resources::light_grid::LightGrid;
use unlight_core::types::light::LightFieldData;
use unlight_core::types::prebaked_lighting_data::{
    LightInfo, PrebakedLightingData, WaveEdge, WaveEdgeData,
};
use unmission_core::events::{LevelReadyEvent, MapGeometryInitializedEvent};
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;

pub fn init_light_grid(
    mut lg: If<ResMut<LightGrid>>,
    mut ev: MessageReader<MapGeometryInitializedEvent>,
) {
    for ev in ev.read() {
        lg.light_field = Array3::from_elem(ev.map_size, LightFieldData::default());
        lg.exposure = unlight_core::exposure::ExposureModel::new();
    }
}

pub fn reset_light_grid(mut lg: If<ResMut<LightGrid>>) {
    lg.reset();
}

/// System to rebuild the entire lighting field based on prebaked data and active sources.
/// Triggered by BoardTopologyToRebuild events.
pub fn rebuild_lighting_field(
    bf: If<Res<BoardTopology>>,
    bcf: If<Res<BoardCollisionField>>,
    mut lg: If<ResMut<LightGrid>>,
    mut ev_bdr: MessageReader<BoardTopologyToRebuild>,
    qt: Query<(&Position, &Behavior)>,
    mut avg_time: Local<(f32, f32)>,
) {
    // Check if we need to rebuild lighting
    let mut should_rebuild = false;
    for ev in ev_bdr.read() {
        if ev.lighting {
            should_rebuild = true;
            break;
        }
    }

    if !should_rebuild {
        return;
    }

    let build_start_time = Instant::now();

    // Initialize with a dark color or base ambient light
    let mut lfs = Array3::<LightFieldData>::default(bf.map_size);

    // Identify active light sources
    let active_source_ids = identify_active_light_sources(&bf, &lg, &qt);

    // Apply prebaked contributions from active sources
    apply_prebaked_contributions(&active_source_ids, &bf, &lg, &mut lfs);

    // Initial propagation from prebaked wave edges
    propagate_from_wave_edges(&bf, &bcf, &lg, &mut lfs, &active_source_ids);

    // Process stairs to propagate light between floors
    let stair_edges = create_stair_wave_edges(&bf, &bcf, &lfs);

    // If we have stair edges, propagate from them too
    if !stair_edges.is_empty() {
        // Temporarily swap wave edges in LightGrid to use the stair ones
        let original_edges = lg.prebaked_wave_edges.clone();
        lg.prebaked_wave_edges = stair_edges;

        // Propagate from the stairs (using dummy source ID 0)
        let _stair_count =
            propagate_from_wave_edges(&bf, &bcf, &lg, &mut lfs, &vec![0].into_iter().collect());

        // Restore original wave edges
        lg.prebaked_wave_edges = original_edges;
    }

    // Apply ambient light to walls
    apply_ambient_light_to_walls(&bf, &bcf, &mut lfs);

    // Diagnostic check for abrupt light cut-offs
    let mut failure_indices = Vec::new();
    let directions = [(0, 1, 0), (1, 0, 0), (0, -1, 0), (-1, 0, 0)];
    for ((i, j, k), data) in lfs.indexed_iter() {
        if data.lux > 0.1 && bcf.0.0[(i, j, k)].see_through {
            let mut failed = false;
            for &(dx, dy, dz) in &directions {
                let ni = i as i64 + dx;
                let nj = j as i64 + dy;
                let nk = k as i64 + dz;
                if is_in_bounds((ni, nj, nk), bf.map_size) {
                    let n_idx = (ni as usize, nj as usize, nk as usize);
                    if bcf.0.0[n_idx].see_through {
                        let n_lux = lfs[n_idx].lux;
                        if n_lux < data.lux / 10.0 || n_lux > data.lux * 10.0 {
                            failed = true;
                            break;
                        }
                    }
                }
            }
            if failed {
                failure_indices.push((i, j, k));
            }
        }
    }
    if !failure_indices.is_empty() {
        error!(
            "Lighting consistency check failed at {} points",
            failure_indices.len()
        );
        for idx in failure_indices {
            lfs[idx].lux += 50.0;
            lfs[idx].color = (1.0, 0.0, 0.0); // Highlight in Red
        }
    }

    // Update final exposure and stats
    update_exposure_and_stats(&bf, &mut lg, &lfs);

    let total_time = build_start_time.elapsed().as_secs_f32();
    let tot_cnt = 4.0;
    avg_time.0 = (avg_time.0 * avg_time.1 + total_time * tot_cnt) / (avg_time.1 + tot_cnt);
    avg_time.1 += 1.0;
}

/// System to prebake lighting when the level is ready.
pub fn prebake_lighting_on_level_ready(
    bf: Res<BoardTopology>,
    mut bcf: ResMut<BoardCollisionField>,
    mut olg: Option<ResMut<LightGrid>>,
    mut ev: MessageReader<LevelReadyEvent>,
    qt: Query<(Entity, &Position, &Behavior)>,
) {
    for _ in ev.read() {
        let lg = olg.as_mut().expect("Prebake: LightGrid is mandatory on receiving the LevelReady event - otherwise we can't prebake lights");
        unrender_std::utils::collision::rebuild_collision_data(&bf, &mut bcf, &qt);
        prebake_lighting_field(&bf, &bcf, lg, &qt);
    }
}

/// Computes the prebaked lighting field for a map.
pub fn prebake_lighting_field(
    bf: &BoardTopology,
    bcf: &BoardCollisionField,
    lg: &mut LightGrid,
    qt: &Query<(Entity, &Position, &Behavior)>,
) {
    debug!("Computing prebaked lighting field...");
    let build_start_time = Instant::now();

    // Create a new Array3 with default values
    let mut prebaked = Array3::from_elem(bf.map_size, PrebakedLightingData::default());

    // First pass - identify all light sources and assign unique IDs
    let mut light_source_count = 0;
    let mut next_source_id = 1; // Start from 1, 0 is reserved for "no source"

    lg.prebaked_metadata = Default::default();
    // Process all entities to find light sources
    for (entity, pos, behavior) in qt.iter() {
        let board_pos = pos.to_board_position();
        let idx = board_pos.ndidx();

        if behavior.p.is_door {
            lg.prebaked_metadata.doors.push(entity);
        }

        if behavior.p.is_breaker {
            lg.prebaked_metadata.breakers.push(entity);
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
            lg.prebaked_metadata
                .light_source_ids
                .insert(entity, next_source_id);
            lg.prebaked_metadata.light_sources.push((entity, idx));
            next_source_id += 1;
        }
    }

    debug!("Prebaking - Found {} light sources", light_source_count);
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
                2.0, // Distance travelled by light (in tiles)
                initial_history,
            ));

            // Mark source position as visited
            let source_visited = visited_by_source.entry(source_id).or_default();
            source_visited.insert((i as i64, j as i64, k as i64));
        }
    }

    // Track statistics
    let mut _propagated_tiles = 0;
    lg.prebaked_wave_edges = Vec::new();

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
            let collision = &bcf.0[neighbor_idx];

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
                lg.prebaked_wave_edges.push(WaveEdgeData {
                    position: pos.clone(),
                    source_id,
                    lux: src_light_lux / (distance_travelled * distance_travelled),
                    color,
                    wave_edge: WaveEdge {
                        src_light_lux,
                        distance_travelled,
                        current_pos: (pos.x as f32, pos.y as f32, pos.z as f32),
                        iir_mean_pos: (pos_mid.x as f32, pos_mid.y as f32, pos_mid.z as f32),
                        iir_mean_iir_mean_pos: (
                            pos_last.x as f32,
                            pos_last.y as f32,
                            pos_last.z as f32,
                        ),
                    },
                });

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

                _propagated_tiles += 1;
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

    // Create a HashSet of all source IDs
    let _all_source_ids: HashSet<u32> = visited_by_source.keys().copied().collect();

    // Store the prebaked data in LightGrid
    lg.prebaked_lighting = prebaked;

    // Call prebake_propagation_data
    prebake_propagation_data(bf, bcf, lg);

    debug!(
        "Prebaked lighting field computed in: {:?}",
        build_start_time.elapsed()
    );
}

/// Pre-computes the allowed propagation directions for each light source and tile
fn prebake_propagation_data(bf: &BoardTopology, bcf: &BoardCollisionField, lg: &mut LightGrid) {
    let map_size = bf.map_size;

    // Create and initialize the vector of Array3
    lg.prebaked_propagation =
        vec![
            Array3::from_elem((map_size.0, map_size.1, map_size.2), [false; 4]);
            lg.prebaked_metadata.light_sources.len() + 1
        ];

    // Second pass - compute allowed propagation directions for each light source
    for (source_entity, source_idx) in &lg.prebaked_metadata.light_sources {
        let source_id = match lg.prebaked_metadata.light_source_ids.get(source_entity) {
            Some(id) => *id,
            None => continue,
        };

        let source_pos = BoardPosition::from_ndidx(*source_idx);

        // Initialize distance field with f32::INFINITY
        let mut distance_field =
            Array3::from_elem((map_size.0, map_size.1, map_size.2), f32::INFINITY);
        distance_field[source_pos.ndidx()] = 0.0;

        let mut queue = VecDeque::new();
        queue.push_front(source_pos.clone());

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
                    None => continue,
                };
                let collision = &bcf.0[n_idx];

                // Skip if this is a static obstacle (except doors)
                if !collision.see_through && !collision.is_dynamic {
                    continue;
                }

                let new_distance = current_distance + 1.0;

                // Only update if this is a shorter path
                if new_distance < distance_field[n_idx] {
                    distance_field[n_idx] = new_distance;

                    // Mark that we can propagate from pos in direction dir_idx
                    lg.prebaked_propagation[source_id as usize][pos.ndidx()][dir_idx] = true;

                    queue.push_front(neighbor_pos.clone());
                }
            }
        }
    }
}
