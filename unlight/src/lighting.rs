use crate::cached_board_pos::CachedBoardPos;
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet, Instant},
};
use fastapprox::faster;
use ndarray::Array3;
use std::{collections::VecDeque, time::Duration};
use uncore::{
    behavior::{Behavior, Class, Orientation, TileState},
    components::board::{boardposition::BoardPosition, position::Position},
    resources::board_data::BoardData,
    types::board::{
        fielddata::LightFieldData,
        prebaked_lighting_data::{
            LightInfo, PendingPropagation, PrebakedLightingData, PropagationDirection,
            SourceContribution,
        },
    },
};

const USE_NEW_LIGHTING: bool = true;

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
pub fn rebuild_lighting_field(bf: &mut BoardData, qt: &Query<(&Position, &Behavior)>) {
    if USE_NEW_LIGHTING {
        info!("Using new rebuild_lighting_field_new");
        rebuild_lighting_field_new(bf, qt);
    } else {
        info!("Using legacy rebuild_lighting_field_old");
        rebuild_lighting_field_old(bf, qt);
    }
}

/// Legacy implementation stub for rebuilding the lighting field.
pub fn rebuild_lighting_field_old(bf: &mut BoardData, qt: &Query<(&Position, &Behavior)>) {
    // NOTE: This is the legacy implementation stub.
    info!("Called legacy rebuild_lighting_field_old");
    let build_start_time = Instant::now();
    let cbp = CachedBoardPos::new();
    bf.exposure_lux = 1.0;
    if bf.light_field.dim() != bf.map_size {
        return;
    }
    let mut lfs = Array3::from_elem(bf.map_size, LightFieldData::default());

    let def_light = LightFieldData::default();
    for (pos, behavior) in qt.iter() {
        let pos = pos.to_board_position();
        lfs[pos.ndidx()] = LightFieldData {
            lux: behavior.p.light.emmisivity_lumens() + def_light.lux,
            color: behavior.p.light.color(),
            transmissivity: behavior.p.light.transmissivity_factor() * def_light.transmissivity
                + 0.0001,
            additional: def_light
                .additional
                .add(&behavior.p.light.additional_data()),
        };
    }
    warn!("Map Size: {:?}", bf.map_size);

    for step in 0..3 {
        let src_lfs = lfs.clone();

        // lfs_clone_time_total += lfs_clone_time.elapsed();
        let size = match step {
            0 => 26,
            1 => 6,
            2 => 3,
            3 => 3,
            _ => 6,
        };
        let mut total_time1 = Duration::default();
        let mut total_time2 = Duration::default();

        for (root_ndidx, src) in src_lfs.indexed_iter() {
            let root_pos = BoardPosition {
                x: root_ndidx.0 as i64,
                y: root_ndidx.1 as i64,
                z: root_ndidx.2 as i64,
            };
            let mut src_lux = src.lux;
            let min_lux = match step {
                0 => 0.001,
                1 => 0.000001,
                _ => 0.0000000001,
            };
            let max_lux = match step {
                0 => f32::MAX,
                1 => 10000.0,
                2 => 1000.0,
                3 => 0.1,
                _ => 0.01,
            };
            if src_lux < min_lux {
                continue;
            }
            if src_lux > max_lux {
                continue;
            }

            if step > 0 {
                // Optimize next steps by only looking to harsh differences.
                let nbors = root_pos.iter_xy_neighbors(1, bf.map_size);
                let ldata_iter = nbors.map(|b| {
                    let l = &lfs[b.ndidx()];
                    (
                        ordered_float::OrderedFloat(l.lux),
                        ordered_float::OrderedFloat(l.transmissivity),
                    )
                });
                let mut min_lux = ordered_float::OrderedFloat(f32::MAX);
                let mut max_lux = ordered_float::OrderedFloat(0.0);
                let mut min_trans = ordered_float::OrderedFloat(2.0);
                for (lux, trans) in ldata_iter {
                    min_lux = min_lux.min(lux);
                    max_lux = max_lux.max(lux);
                    min_trans = min_trans.min(trans);
                }

                // For smoothing steps only:
                if *max_lux / (*min_lux + 0.0001) < 1.2 {
                    continue;
                }
                if *min_trans > 0.7 && src_lux / (*min_lux + 0.0001) < 1.9 {
                    // If there are no walls nearby, we don't reflect light.
                    continue;
                }
            }

            // This controls how harsh is the light
            if step > 0 {
                src_lux /= 5.5;
            } else {
                src_lux /= 1.01;
            }

            // let shadows_time = Instant::now();
            let nbors = root_pos.iter_xy_neighbors(size, bf.map_size);
            lfs[root_ndidx].lux -= src_lux;
            let mut shadow_dist = [(size + 1) as f32; CachedBoardPos::TAU_I];

            let time1 = bevy::utils::Instant::now();

            // Instead of iterating over nbors.clone(), compute a contiguous region around root_pos.
            let root_x = root_ndidx.0;
            let root_y = root_ndidx.1;
            let board_x = root_x.saturating_sub(size as usize)
                ..(root_x + size as usize + 1).min(bf.map_size.0);
            let board_y = root_y.saturating_sub(size as usize)
                ..(root_y + size as usize + 1).min(bf.map_size.1);

            // Create a view into the light-field for this window (make sure to choose the correct z value)
            let window = lfs.slice(ndarray::s![board_x.clone(), board_y.clone(), root_ndidx.2]);

            // Now get the cache slices
            let dist_view = cbp.dist_slice(&root_pos, board_x.clone(), board_y.clone());
            let angle_view = cbp.angle_slice(&root_pos, board_x.clone(), board_y.clone());
            let angle_range_view = cbp.angle_range_slice(&root_pos, board_x, board_y);

            // Then iterate over the indices of the window:
            ndarray::Zip::indexed(&window)
                .and(&dist_view)
                .and(&angle_view)
                .for_each(|(i, j), lf, &cached_dist, &cached_angle| {
                    if lf.transmissivity >= 0.5 {
                        return;
                    }
                    let angle_range = angle_range_view[(i, j)];
                    for d in angle_range.0..=angle_range.1 {
                        let ang = (cached_angle as i64 + d).rem_euclid(CachedBoardPos::TAU_I as i64)
                            as usize;
                        shadow_dist[ang] = shadow_dist[ang].min(cached_dist);
                    }
                });
            total_time1 += time1.elapsed();

            // shadows_time_total += shadows_time.elapsed(); FIXME: Possibly we want to smooth
            // shadow_dist here - a convolution with a gaussian or similar where we preserve
            // the high values but smooth the transition to low ones.
            if src.transmissivity < 0.5 {
                // Reduce light spread through walls
                shadow_dist.iter_mut().for_each(|x| *x = 0.0);
            }

            // let size = shadow_dist .iter() .map(|d| (d + 1.5).round() as u32) .max()
            // .unwrap() .min(size); let nbors = root_pos.xy_neighbors(size);
            let light_height = 4.0;

            // let mut total_lux = 0.1; for neighbor in nbors.iter() { let dist =
            // cbp.bpos_dist(&root_pos, neighbor); let dist2 = dist + light_height; let angle
            // = cbp.bpos_angle(&root_pos, neighbor); let sd = shadow_dist[angle]; let f =
            // (faster::tanh(sd - dist - 0.5) + 1.0) / 2.0; total_lux += f / dist2 / dist2; }
            // let store_lfs_time = Instant::now();
            let total_lux = 2.0;
            let time2 = bevy::utils::Instant::now();

            // new shadow method
            for neighbor in nbors {
                let dist = cbp.bpos_dist(&root_pos, &neighbor);

                // let dist = root_pos.fast_distance_xy(neighbor);
                let dist2 = dist + light_height;
                let angle = cbp.bpos_angle(&root_pos, &neighbor);
                let sd = shadow_dist[angle];
                let lux_add = src_lux / dist2 / dist2 / total_lux;
                if dist - 3.0 < sd {
                    // FIXME: f here controls the bleed through walls.
                    // 0.5 is too low, it creates un-evenness.
                    const BLEED_TILES: f32 = 0.8;
                    let f = (faster::tanh((sd - dist - 0.5) * BLEED_TILES.recip()) + 1.0) / 2.0;

                    // let f = 1.0;
                    lfs[neighbor.ndidx()].lux += lux_add * f;
                }
            }
            total_time2 += time2.elapsed();
            // store_lfs_time_total += store_lfs_time.elapsed();
        }
        warn!("Time to compute shadows: {step} - {:?}", total_time1);
        warn!("Time to store lfs: {step} - {:?}", total_time2);
        // info!( "Light step {}: {:?}; per size: {:?}", step, step_time.elapsed(),
        // step_time.elapsed() / size );
    }

    // let's get an average of lux values
    let total_lux: f32 = lfs.iter().map(|x| x.lux).sum();
    let count = (bf.map_size.0 * bf.map_size.1 * bf.map_size.2) as f32;
    let avg_lux = total_lux / count;
    bf.exposure_lux = (avg_lux + 2.0) / 2.0;
    bf.light_field = lfs;

    // dbg!(lfs_clone_time_total); dbg!(shadows_time_total);
    // dbg!(store_lfs_time_total);
    info!(
        "Lighting rebuild - complete: {:?}",
        build_start_time.elapsed()
    );
}

/// Rebuilds the lighting field using prebaked static propagation data
pub fn rebuild_lighting_field_new(bf: &mut BoardData, qt: &Query<(&Position, &Behavior)>) {
    info!("Starting rebuild_lighting_field_new using prebaked data");
    let build_start_time = Instant::now();

    // Create a new light field with default values
    let mut lfs = Array3::from_elem(bf.map_size, LightFieldData::default());

    // 1. Identify active light sources
    let (active_source_ids, dynamic_lights) = identify_active_light_sources(bf, qt);

    // 2. Apply prebaked contributions from active sources
    let initial_tiles_lit = apply_prebaked_contributions(&active_source_ids, bf, &mut lfs);

    // 3. Handle door states and identify continuation points
    let door_states = collect_door_states(qt);
    let portal_points = identify_portal_points(bf);
    let continuation_points =
        prepare_continuation_points(bf, &active_source_ids, &door_states, &portal_points);

    // 4. Add dynamic lights to initial state
    let mut visited = add_dynamic_light_sources(bf, &mut lfs, dynamic_lights);

    // 5. Propagate light from continuation points
    let (dynamic_propagation_count, light_continued) = propagate_from_continuation_points(
        bf,
        &mut lfs,
        &mut visited,
        &continuation_points,
        &active_source_ids,
    );

    info!(
        "Dynamic BFS propagation: {} additional light propagations, {:.2} total lux continued, {} initial tiles lit",
        dynamic_propagation_count, light_continued, initial_tiles_lit
    );

    // 6. Apply ambient light to walls
    apply_ambient_light_to_walls(bf, &mut lfs);

    // 7. Calculate exposure and update board data
    update_exposure_and_stats(bf, &lfs);

    info!(
        "BFS light propagation completed in: {:?}",
        build_start_time.elapsed()
    );
}

// Applies ambient light to walls based on neighboring lit tiles
fn apply_ambient_light_to_walls(bf: &BoardData, lfs: &mut Array3<LightFieldData>) {
    let wall_light_start = Instant::now();
    let mut walls_lit = 0;

    // Define directions for 4-way connectivity
    let directions = [(0, 1, 0), (1, 0, 0), (0, -1, 0), (-1, 0, 0)];

    // Threshold for considering a tile "dark"
    const DARK_THRESHOLD: f32 = 0.000001;

    for ((i, j, k), collision) in bf.collision_field.indexed_iter() {
        // Only process dark tiles
        if lfs[(i, j, k)].lux > DARK_THRESHOLD {
            continue;
        }

        // Collect light from neighbors
        let mut total_lux = 0.0;
        let mut weighted_color_sum = (0.0, 0.0, 0.0);
        let mut weight_sum = 0.0;

        for &(dx, dy, dz) in &directions {
            let nx = i as i64 + dx;
            let ny = j as i64 + dy;
            let nz = k as i64 + dz;

            // Skip if out of bounds
            if !is_in_bounds((nx, ny, nz), bf.map_size) {
                continue;
            }

            let n_pos = (nx as usize, ny as usize, nz as usize);
            let neighbor_light = &lfs[n_pos];

            // Skip if neighbor has no light
            if neighbor_light.lux <= 0.001 {
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
            };

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
            lfs[(i, j, k)].lux = total_lux;
            lfs[(i, j, k)].color = avg_color;
            walls_lit += 1;
        }
    }

    info!(
        "Wall ambient light pass: {} walls lit in {:?}",
        walls_lit,
        wall_light_start.elapsed()
    );
}

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
        let lux = behavior.p.light.emmisivity_lumens();
        if lux > 0.0 {
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
    let transmissivity_zero_count = prebaked
        .iter()
        .filter(|d| d.light_info.transmissivity == 0.0)
        .count();
    let transmissivity_gt_one_count = prebaked
        .iter()
        .filter(|d| d.light_info.transmissivity >= 1.0)
        .count();
    let total_tiles = bf.map_size.0 * bf.map_size.1 * bf.map_size.2;

    info!(
        "Transmissivity stats: {}/{} tiles are opaque (0.0), {}/{} tiles allow light (>=1.0)",
        transmissivity_zero_count, total_tiles, transmissivity_gt_one_count, total_tiles
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
                if new_lux < 0.001 {
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

/// Helper function to check if there are active light sources nearby
#[allow(dead_code)]
fn has_active_light_nearby(
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

                if prebaked_data.light_info.is_source
                    && active_source_ids.contains(&prebaked_data.light_info.source_id)
                {
                    return true;
                }
            }
        }
    }

    false
}

/// Determines if a light is currently active based on its position and behavior
#[allow(dead_code)]
fn is_light_active(pos: &BoardPosition, behaviors: &HashMap<BoardPosition, &Behavior>) -> bool {
    if let Some(behavior) = behaviors.get(pos) {
        behavior.p.light.emits_light
    } else {
        false
    }
}

/// Checks if a position is within the board boundaries
fn is_in_bounds(pos: (i64, i64, i64), map_size: (usize, usize, usize)) -> bool {
    pos.0 >= 0
        && pos.1 >= 0
        && pos.2 >= 0
        && pos.0 < map_size.0 as i64
        && pos.1 < map_size.1 as i64
        && pos.2 < map_size.2 as i64
}

/// Gets the relative neighbor position from a direction
#[allow(dead_code)]
fn get_direction_offset(dir: PropagationDirection) -> (i64, i64, i64) {
    match dir {
        PropagationDirection::North => (0, 1, 0),
        PropagationDirection::East => (1, 0, 0),
        PropagationDirection::South => (0, -1, 0),
        PropagationDirection::West => (-1, 0, 0),
    }
}

/// Checks if light can propagate in a specific direction from the given position
#[allow(dead_code)]
fn can_propagate_in_direction(
    prebaked_data: &PrebakedLightingData,
    direction: PropagationDirection,
) -> bool {
    match direction {
        PropagationDirection::North => prebaked_data.propagation_dirs.north,
        PropagationDirection::East => prebaked_data.propagation_dirs.east,
        PropagationDirection::South => prebaked_data.propagation_dirs.south,
        PropagationDirection::West => prebaked_data.propagation_dirs.west,
    }
}

/// Blend two colors based on their intensity
fn blend_colors(c1: (f32, f32, f32), lux1: f32, c2: (f32, f32, f32), lux2: f32) -> (f32, f32, f32) {
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
fn identify_active_light_sources(
    bf: &BoardData,
    qt: &Query<(&Position, &Behavior)>,
) -> (
    HashSet<u32>,
    Vec<(BoardPosition, f32, (f32, f32, f32), f32)>,
) {
    let mut active_source_ids = HashSet::new();
    let mut dynamic_lights = Vec::new();

    // Create a map of entity positions to their behaviors
    let mut position_to_behavior = HashMap::new();
    for (pos, behavior) in qt.iter() {
        position_to_behavior.insert(pos.to_board_position(), behavior);
    }

    // First pass: mark prebaked sources
    for ((i, j, k), prebaked_data) in bf.prebaked_lighting.indexed_iter() {
        if prebaked_data.light_info.is_source {
            let pos = BoardPosition {
                x: i as i64,
                y: j as i64,
                z: k as i64,
            };

            // Check if this light source is currently emitting light
            if let Some(behavior) = position_to_behavior.get(&pos) {
                if behavior.p.light.emits_light {
                    active_source_ids.insert(prebaked_data.light_info.source_id);
                }
            }
        }
    }

    // Collect dynamic lights that aren't in prebaked data
    for (pos, behavior) in qt.iter() {
        let board_pos = pos.to_board_position();
        let idx = board_pos.ndidx();
        let lux = behavior.p.light.emmisivity_lumens();

        if lux > 0.0 && behavior.p.light.emits_light {
            let prebaked_data = &bf.prebaked_lighting[idx];
            if !prebaked_data.light_info.is_source {
                dynamic_lights.push((
                    board_pos.clone(),
                    lux,
                    behavior.p.light.color(),
                    30.0, // Default maximum propagation distance
                ));
            }
        }
    }

    info!(
        "Active light sources: {}/{} (prebaked) + {} dynamic sources",
        active_source_ids.len(),
        bf.prebaked_lighting
            .iter()
            .filter(|d| d.light_info.is_source)
            .count(),
        dynamic_lights.len()
    );

    (active_source_ids, dynamic_lights)
}

/// Apply prebaked light contributions from active sources
#[allow(dead_code)]
fn apply_prebaked_contributions(
    active_source_ids: &HashSet<u32>,
    bf: &BoardData,
    lfs: &mut Array3<LightFieldData>,
) -> usize {
    let mut tiles_lit = 0;

    for ((i, j, k), prebaked_data) in bf.prebaked_lighting.indexed_iter() {
        let pos_idx = (i, j, k);
        if prebaked_data.source_contributions.is_empty() {
            continue;
        }

        // Optimization for common case (few light sources)
        if prebaked_data.source_contributions.len() <= 4 {
            let mut total_lux = 0.0;
            let mut weighted_color = (0.0, 0.0, 0.0);

            for (source_id, contribution) in &prebaked_data.source_contributions {
                // Fix the HashSet contains issue by dereferencing the source_id
                if active_source_ids.contains(source_id) {
                    total_lux += contribution.lux;
                    weighted_color.0 += contribution.color.0 * contribution.lux;
                    weighted_color.1 += contribution.color.1 * contribution.lux;
                    weighted_color.2 += contribution.color.2 * contribution.lux;
                }
            }

            if total_lux > 0.0 {
                tiles_lit += 1;
                lfs[pos_idx] = LightFieldData {
                    lux: total_lux,
                    color: (
                        weighted_color.0 / total_lux,
                        weighted_color.1 / total_lux,
                        weighted_color.2 / total_lux,
                    ),
                    transmissivity: prebaked_data.light_info.transmissivity,
                    additional: Default::default(),
                };
            }
        } else {
            // Handle larger collections - fix the HashSet contains issue
            let active_contributions: Vec<_> = prebaked_data
                .source_contributions
                .iter()
                .filter(|&(id, _)| active_source_ids.contains(id))
                .collect();

            if !active_contributions.is_empty() {
                let total_lux: f32 = active_contributions.iter().map(|(_, c)| c.lux).sum();

                if total_lux > 0.0 {
                    tiles_lit += 1;
                    let mut weighted_color = (0.0, 0.0, 0.0);

                    for (_, contribution) in &active_contributions {
                        let weight = contribution.lux / total_lux;
                        weighted_color.0 += contribution.color.0 * weight;
                        weighted_color.1 += contribution.color.1 * weight;
                        weighted_color.2 += contribution.color.2 * weight;
                    }

                    lfs[pos_idx] = LightFieldData {
                        lux: total_lux,
                        color: weighted_color,
                        transmissivity: prebaked_data.light_info.transmissivity,
                        additional: Default::default(),
                    };
                }
            }
        }
    }

    info!("Applied prebaked contributions: {} tiles lit", tiles_lit);
    tiles_lit
}

/// Update final exposure settings and log statistics
fn update_exposure_and_stats(bf: &mut BoardData, lfs: &Array3<LightFieldData>) {
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

/// Identifies portal points for special light propagation
fn identify_portal_points(bf: &BoardData) -> HashSet<(usize, usize, usize)> {
    let mut portal_points = HashSet::new();

    for ((i, j, k), prebaked_data) in bf.prebaked_lighting.indexed_iter() {
        if !prebaked_data.is_continuation_point {
            continue;
        }

        // Check for corridors and corners by examining propagation directions
        let props = &prebaked_data.propagation_dirs;

        // Horizontal corridor
        let east_west = props.east && props.west && !props.north && !props.south;

        // Vertical corridor
        let north_south = props.north && props.south && !props.east && !props.west;

        // Corner cases
        let north_east = props.north && props.east && !props.south && !props.west;
        let north_west = props.north && props.west && !props.south && !props.east;
        let south_east = props.south && props.east && !props.north && !props.west;
        let south_west = props.south && props.west && !props.north && !props.east;

        if north_south || east_west || north_east || north_west || south_east || south_west {
            portal_points.insert((i, j, k));
        }
    }

    info!("Identified {} portal points", portal_points.len());
    portal_points
}

/// Collects information about door states from entity behaviors
fn collect_door_states(qt: &Query<(&Position, &Behavior)>) -> HashMap<(usize, usize, usize), bool> {
    let mut door_states = HashMap::new();

    for (pos, behavior) in qt.iter() {
        // Check if this entity is a door
        let is_door = behavior.key_cvo().class == Class::Door;

        if is_door {
            let board_pos = pos.to_board_position();
            let idx = (
                board_pos.x as usize,
                board_pos.y as usize,
                board_pos.z as usize,
            );
            let is_open = behavior.state() == TileState::Open;

            // Store the door's open state (true if open, false if closed)
            door_states.insert(idx, is_open);
        }
    }

    info!("Collected {} door states", door_states.len());
    door_states
}

/// Prepares continuation points for dynamic light propagation
fn prepare_continuation_points(
    bf: &BoardData,
    active_source_ids: &HashSet<u32>,
    door_states: &HashMap<(usize, usize, usize), bool>,
    portal_points: &HashSet<(usize, usize, usize)>,
) -> Vec<(BoardPosition, u32, f32, f32, (f32, f32, f32))> {
    let mut continuation_points = Vec::new();

    // Check all prebaked data for pending propagations
    for ((i, j, k), prebaked_data) in bf.prebaked_lighting.indexed_iter() {
        // Skip if not a continuation point or has no pending propagations
        if !prebaked_data.is_continuation_point || prebaked_data.pending_propagations.is_empty() {
            continue;
        }

        let pos_idx = (i, j, k);
        let pos = BoardPosition {
            x: i as i64,
            y: j as i64,
            z: k as i64,
        };

        // Check if this is a door and if it's open
        let is_door_position = door_states.contains_key(&pos_idx);
        let is_open_door = door_states.get(&pos_idx).copied().unwrap_or(false);

        // Check if this is a portal point
        let is_portal = portal_points.contains(&pos_idx);

        // KEY FIX: Only continue light propagation from this point if:
        // 1. It's a door and it's currently open (door state == open)
        // 2. OR it's a portal point and not a door (for other continuation points)
        if (is_door_position && is_open_door) || (!is_door_position && is_portal) {
            // For each pending propagation from an active light source
            for pending in &prebaked_data.pending_propagations {
                // Skip if source is not active
                if !active_source_ids.contains(&pending.source_id) {
                    continue;
                }

                // Add to continuation points with the pending propagation data
                continuation_points.push((
                    pos.clone(),
                    pending.source_id,
                    pending.remaining_distance,
                    pending.incoming_lux,
                    pending.color,
                ));
            }
        }
    }

    info!("Prepared {} continuation points", continuation_points.len());
    continuation_points
}

/// Adds dynamic light sources to the lighting field
fn add_dynamic_light_sources(
    bf: &BoardData,
    lfs: &mut Array3<LightFieldData>,
    dynamic_lights: Vec<(BoardPosition, f32, (f32, f32, f32), f32)>,
) -> Array3<bool> {
    let mut visited = Array3::from_elem(bf.map_size, false);
    let mut dynamic_queue = VecDeque::new();

    // Add all dynamic light sources to the queue
    for (pos, lux, color, distance) in dynamic_lights {
        let idx = pos.ndidx();

        // Update light field with dynamic source
        lfs[idx].lux += lux;
        if lfs[idx].lux > 0.0 {
            lfs[idx].color = blend_colors(lfs[idx].color, lfs[idx].lux - lux, color, lux);
        } else {
            lfs[idx].color = color;
        }

        // Add to queue for propagation
        dynamic_queue.push_back((pos, distance, lux, color));
        visited[idx] = true;
    }

    // Define directions for propagation
    let directions = [(0, 1, 0), (1, 0, 0), (0, -1, 0), (-1, 0, 0)];

    // Propagate dynamic lights
    let mut propagation_count = 0;

    while let Some((pos, remaining_distance, current_lux, color)) = dynamic_queue.pop_front() {
        // Skip if we've reached the distance limit or light is too dim
        if remaining_distance <= 0.0 || current_lux < 0.001 {
            continue;
        }

        // Get propagation directions from prebaked data
        let pos_idx = pos.ndidx();
        let prebaked_data = &bf.prebaked_lighting[pos_idx];

        // Process each direction
        for (dir_i, &(dx, dy, dz)) in directions.iter().enumerate() {
            // Check if can propagate in this direction
            let can_propagate = match dir_i {
                0 => prebaked_data.propagation_dirs.north,
                1 => prebaked_data.propagation_dirs.east,
                2 => prebaked_data.propagation_dirs.south,
                3 => prebaked_data.propagation_dirs.west,
                _ => false,
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

            // Skip if already visited
            if visited[neighbor_idx] {
                continue;
            }

            // Check if light can pass through
            let neighbor_prebaked = &bf.prebaked_lighting[neighbor_idx];
            let collision = &bf.collision_field[neighbor_idx];

            if !collision.see_through {
                continue;
            }

            // Calculate diminished light
            let transmissivity = neighbor_prebaked.light_info.transmissivity;
            let falloff = 0.75 * transmissivity;
            let new_lux = current_lux * falloff;

            // Update light field for neighbor
            lfs[neighbor_idx].lux += new_lux;
            if lfs[neighbor_idx].lux > 0.0 {
                lfs[neighbor_idx].color = blend_colors(
                    lfs[neighbor_idx].color,
                    lfs[neighbor_idx].lux - new_lux,
                    color,
                    new_lux,
                );
            }

            // Add neighbor to queue
            dynamic_queue.push_back((neighbor_pos, remaining_distance - 1.0, new_lux, color));
            visited[neighbor_idx] = true;

            propagation_count += 1;
        }
    }

    info!("Added {} dynamic light propagations", propagation_count);
    visited
}

/// Propagates light from continuation points
fn propagate_from_continuation_points(
    bf: &BoardData,
    lfs: &mut Array3<LightFieldData>,
    visited: &mut Array3<bool>,
    continuation_points: &[(BoardPosition, u32, f32, f32, (f32, f32, f32))],
    active_source_ids: &HashSet<u32>,
) -> (usize, f32) {
    let mut queue = VecDeque::new();
    let mut propagation_count = 0;
    let mut total_light_continued = 0.0;

    // Add all continuation points to the queue
    for &(ref pos, source_id, remaining_distance, lux, color) in continuation_points {
        queue.push_back((pos.clone(), source_id, remaining_distance, lux, color));
        total_light_continued += lux;
    }

    // Define directions for propagation
    let directions = [(0, 1, 0), (1, 0, 0), (0, -1, 0), (-1, 0, 0)];

    // Process queue using BFS
    while let Some((pos, source_id, remaining_distance, current_lux, color)) = queue.pop_front() {
        // Skip if we've reached the distance limit or light is too dim
        if remaining_distance <= 0.0 || current_lux < 0.001 {
            continue;
        }

        // Skip if source is no longer active
        if !active_source_ids.contains(&source_id) {
            continue;
        }

        let pos_idx = pos.ndidx();

        // Get propagation directions from prebaked data
        let prebaked_data = &bf.prebaked_lighting[pos_idx];

        // Process each direction
        for (dir_i, &(dx, dy, dz)) in directions.iter().enumerate() {
            // Check if can propagate in this direction
            let can_propagate = match dir_i {
                0 => prebaked_data.propagation_dirs.north,
                1 => prebaked_data.propagation_dirs.east,
                2 => prebaked_data.propagation_dirs.south,
                3 => prebaked_data.propagation_dirs.west,
                _ => false,
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

            // Skip if already visited
            if visited[neighbor_idx] {
                continue;
            }

            // Check collision data
            let collision = &bf.collision_field[neighbor_idx];
            if !collision.see_through {
                continue;
            }

            // Calculate diminished light
            let neighbor_prebaked = &bf.prebaked_lighting[neighbor_idx];
            let transmissivity = neighbor_prebaked.light_info.transmissivity;
            let falloff = 0.75 * transmissivity;
            let new_lux = current_lux * falloff;

            // Update light field for neighbor
            lfs[neighbor_idx].lux += new_lux;
            if lfs[neighbor_idx].lux > 0.0 {
                lfs[neighbor_idx].color = blend_colors(
                    lfs[neighbor_idx].color,
                    lfs[neighbor_idx].lux - new_lux,
                    color,
                    new_lux,
                );
            } else {
                lfs[neighbor_idx].color = color;
            }

            // Add neighbor to queue
            queue.push_back((
                neighbor_pos,
                source_id,
                remaining_distance - 1.0,
                new_lux,
                color,
            ));
            visited[neighbor_idx] = true;

            propagation_count += 1;
        }
    }

    (propagation_count, total_light_continued)
}
