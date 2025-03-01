use crate::cached_board_pos::CachedBoardPos;
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet, Instant},
};
use fastapprox::faster;
use ndarray::Array3;
use std::{collections::VecDeque, time::Duration};
use uncore::{
    behavior::{Behavior, Orientation},
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

/// Rebuilds the lighting field using prebaked static propagation data as a seed,
/// and then extends the light via BFS.
pub fn rebuild_lighting_field_new(bf: &mut BoardData, qt: &Query<(&Position, &Behavior)>) {
    info!("Starting rebuild_lighting_field_new using prebaked data");
    let build_start_time = Instant::now();

    // Create a new light field with default values
    let mut lfs = Array3::from_elem(bf.map_size, LightFieldData::default());

    // First, we need to determine which light sources are currently active
    // Create a set of active source IDs
    let mut active_source_ids = HashSet::new();

    // Track dynamic lights by collecting them from entities
    for (pos, behavior) in qt.iter() {
        let board_pos = pos.to_board_position();
        let idx = board_pos.ndidx();

        // Check if this tile has a static light source in prebaked data
        let prebaked_data = &bf.prebaked_lighting[idx];
        if prebaked_data.light_info.is_source {
            // Check if the light is currently on
            let is_on = behavior.p.light.emmisivity_lumens() > 0.0;
            if is_on {
                // Add this light source ID to the active set
                active_source_ids.insert(prebaked_data.light_info.source_id);
            }
        }
    }

    info!(
        "Active light sources: {}/{}",
        active_source_ids.len(),
        bf.prebaked_lighting
            .iter()
            .filter(|d| d.light_info.is_source)
            .count()
    );

    // Queue for BFS processing with (position, source_id, remaining_distance, current_lux, color)
    let mut queue = VecDeque::new();

    // Visited tracking - we'll track positions that have been visited by any light
    let mut visited = Array3::from_elem(bf.map_size, false);

    // Apply initial light contributions from active sources
    let mut initial_tiles_lit = 0;

    // First pass: Apply prebaked contributions for all active light sources
    for ((i, j, k), prebaked_data) in bf.prebaked_lighting.indexed_iter() {
        let pos_idx = (i, j, k);
        let mut total_lux = 0.0;
        let mut weighted_color = (0.0, 0.0, 0.0);

        // Apply contributions from all active sources
        for (source_id, contribution) in &prebaked_data.source_contributions {
            if active_source_ids.contains(source_id) {
                total_lux += contribution.lux;
                weighted_color.0 += contribution.color.0 * contribution.lux;
                weighted_color.1 += contribution.color.1 * contribution.lux;
                weighted_color.2 += contribution.color.2 * contribution.lux;
            }
        }

        // If we have any light, update the light field
        if total_lux > 0.0 {
            initial_tiles_lit += 1;

            // Normalize the color based on total lux
            let color = if total_lux > 0.0 {
                (
                    weighted_color.0 / total_lux,
                    weighted_color.1 / total_lux,
                    weighted_color.2 / total_lux,
                )
            } else {
                (1.0, 1.0, 1.0)
            };

            // Set the light field data
            lfs[pos_idx] = LightFieldData {
                lux: total_lux,
                color,
                transmissivity: prebaked_data.light_info.transmissivity,
                ..Default::default()
            };
        }
    }

    info!(
        "Initial light application: {} tiles lit from prebaked sources",
        initial_tiles_lit
    );

    // Second pass: Check for continuation points where doors are now open
    // These are points that were blocked during prebaking but are now unblocked
    let mut continuation_points_processed = 0;

    let directions = [
        (0, 1, 0, PropagationDirection::North),
        (1, 0, 0, PropagationDirection::East),
        (0, -1, 0, PropagationDirection::South),
        (-1, 0, 0, PropagationDirection::West),
    ];

    for ((i, j, k), prebaked_data) in bf.prebaked_lighting.indexed_iter() {
        // Skip if not a continuation point
        if !prebaked_data.is_continuation_point {
            continue;
        }

        // Check if this point has any pending propagations
        if prebaked_data.pending_propagations.is_empty() {
            continue;
        }

        // Get current collision data
        let pos_idx = (i, j, k);
        let collision = &bf.collision_field[pos_idx];

        // If this tile is now see_through (e.g., an open door), process the pending propagations
        if collision.see_through {
            continuation_points_processed += 1;

            // Process each pending propagation
            for pending in &prebaked_data.pending_propagations {
                // Skip if this source isn't active
                if !active_source_ids.contains(&pending.source_id) {
                    continue;
                }

                // Add to the BFS queue to continue propagation
                let pos = BoardPosition {
                    x: i as i64,
                    y: j as i64,
                    z: k as i64,
                };

                // Get the direction to propagate
                let (dx, dy, dz) = match pending.direction {
                    PropagationDirection::North => (0, 1, 0),
                    PropagationDirection::East => (1, 0, 0),
                    PropagationDirection::South => (0, -1, 0),
                    PropagationDirection::West => (-1, 0, 0),
                };

                // Calculate the new position to continue from
                let nx = pos.x + dx;
                let ny = pos.y + dy;
                let nz = pos.z + dz;

                // Skip if out of bounds
                if nx < 0
                    || ny < 0
                    || nz < 0
                    || nx >= bf.map_size.0 as i64
                    || ny >= bf.map_size.1 as i64
                    || nz >= bf.map_size.2 as i64
                {
                    continue;
                }

                let new_pos = BoardPosition {
                    x: nx,
                    y: ny,
                    z: nz,
                };

                // Add this position to the BFS queue
                // (position, source_id, remaining_distance, current_lux, color)
                queue.push_back((
                    new_pos.clone(),
                    pending.source_id,
                    pending.remaining_distance - 1.0, // Reduce distance by 1 for the step
                    pending.incoming_lux * 0.75,      // Apply some falloff
                    pending.color,
                ));

                // Mark this position as visited
                visited[new_pos.ndidx()] = true;
            }
        }
    }

    info!(
        "Continuation points processed: {}",
        continuation_points_processed
    );

    // BFS processing - extend light waves from continuation points
    let mut dynamic_propagation_count = 0;

    while let Some((pos, source_id, remaining_distance, current_lux, color)) = queue.pop_front() {
        // Skip if we've reached the distance limit or light is too dim
        if remaining_distance <= 0.0 || current_lux < 0.001 {
            continue;
        }

        let pos_idx = pos.ndidx();

        // Skip if this source isn't active
        if !active_source_ids.contains(&source_id) {
            continue;
        }

        // Add light to the current position
        lfs[pos_idx].lux += current_lux;

        // Blend colors weighted by intensity
        let current_color = lfs[pos_idx].color;
        let total_lux = lfs[pos_idx].lux;

        if total_lux > 0.0 {
            lfs[pos_idx].color = (
                (current_color.0 * (total_lux - current_lux) + color.0 * current_lux) / total_lux,
                (current_color.1 * (total_lux - current_lux) + color.1 * current_lux) / total_lux,
                (current_color.2 * (total_lux - current_lux) + color.2 * current_lux) / total_lux,
            );
        } else {
            lfs[pos_idx].color = color;
        }

        // Get the prebaked data for this position
        let prebaked_data = &bf.prebaked_lighting[pos_idx];

        // Propagate in each direction
        for &(dx, dy, dz, dir) in &directions {
            // Check if the current position allows propagation in this direction
            let can_propagate = match dir {
                PropagationDirection::North => prebaked_data.propagation_dirs.north,
                PropagationDirection::East => prebaked_data.propagation_dirs.east,
                PropagationDirection::South => prebaked_data.propagation_dirs.south,
                PropagationDirection::West => prebaked_data.propagation_dirs.west,
            };

            if !can_propagate {
                continue;
            }

            // Calculate neighbor position
            let nx = pos.x + dx;
            let ny = pos.y + dy;
            let nz = pos.z + dz;

            // Skip if out of bounds
            if nx < 0
                || ny < 0
                || nz < 0
                || nx >= bf.map_size.0 as i64
                || ny >= bf.map_size.1 as i64
                || nz >= bf.map_size.2 as i64
            {
                continue;
            }

            let neighbor_pos = BoardPosition {
                x: nx,
                y: ny,
                z: nz,
            };
            let neighbor_idx = neighbor_pos.ndidx();

            // Check for doors/walls (dynamic check)
            let collision = &bf.collision_field[neighbor_idx];
            if !collision.see_through {
                continue;
            }

            // Check if already visited
            if visited[neighbor_idx] {
                continue; // Wave collision - both waves stop
            }

            // Get the prebaked data for the neighbor
            let neighbor_prebaked = &bf.prebaked_lighting[neighbor_idx];

            // Calculate diminished light level
            let transmissivity = neighbor_prebaked.light_info.transmissivity;
            let falloff = 0.75 * transmissivity;
            let new_lux = current_lux * falloff;

            // Add neighbor to queue for next wave propagation
            queue.push_back((
                neighbor_pos,
                source_id,
                remaining_distance - 1.0,
                new_lux,
                color,
            ));

            // Mark as visited
            visited[neighbor_idx] = true;

            dynamic_propagation_count += 1;
        }
    }

    info!(
        "Dynamic BFS propagation: {} additional light propagations",
        dynamic_propagation_count
    );

    // Count how many tiles have light
    let tiles_with_light = lfs.iter().filter(|x| x.lux > 0.0).count();
    let total_tiles = bf.map_size.0 * bf.map_size.1 * bf.map_size.2;
    let avg_lux = lfs.iter().map(|x| x.lux).sum::<f32>() / total_tiles as f32;
    let max_lux = lfs.iter().map(|x| x.lux).fold(0.0, f32::max);

    info!(
        "Light field after rebuild: {}/{} tiles lit ({:.2}%), avg lux: {:.6}, max lux: {:.6}",
        tiles_with_light,
        total_tiles,
        (tiles_with_light as f32 / total_tiles as f32) * 100.0,
        avg_lux,
        max_lux
    );

    // Apply ambient light to walls based on neighbors
    apply_ambient_light_to_walls(bf, &mut lfs);

    // Calculate exposure
    let total_lux: f32 = lfs.iter().map(|x| x.lux).sum();
    let count = (bf.map_size.0 * bf.map_size.1 * bf.map_size.2) as f32;
    let avg_lux = total_lux / count;
    bf.exposure_lux = (avg_lux + 2.0) / 2.0;

    info!("Final exposure_lux set to: {}", bf.exposure_lux);

    // Update board data with the new light field
    bf.light_field = lfs;

    info!(
        "BFS light propagation completed in: {:?}",
        build_start_time.elapsed()
    );
}

/// Applies ambient light to walls based on neighboring lit tiles
fn apply_ambient_light_to_walls(bf: &BoardData, lfs: &mut Array3<LightFieldData>) {
    let wall_light_start = Instant::now();
    let mut walls_lit = 0;

    // Define directions for 4-way connectivity
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

    // First, identify all walls with no light
    for ((i, j, k), collision) in bf.collision_field.indexed_iter() {
        // FIXME: In here usually we just check if the tile wasn't see through
        // with collision.see_through, and only apply to these, but open doors
        // are currently a problem because once open, they are ommited from lighting
        // by some reason, which could be because of the prebaked data.
        // For now, we'll just apply to all tiles that are suspiciously dark.
        if lfs[(i, j, k)].lux <= 0.000001 {
            // Get orientation directly from the collision field
            let wall_orientation = collision.wall_orientation;

            // Collect light from appropriate neighbors based on orientation
            let mut total_lux = 0.0;
            let mut count = 0;
            let mut avg_color = (0.0, 0.0, 0.0);

            for &(dx, dy, dz) in &directions {
                let nx = i as i64 + dx;
                let ny = j as i64 + dy;
                let nz = k as i64 + dz;

                // Skip if out of bounds
                if nx < 0
                    || ny < 0
                    || nz < 0
                    || nx >= bf.map_size.0 as i64
                    || ny >= bf.map_size.1 as i64
                    || nz >= bf.map_size.2 as i64
                {
                    continue;
                }

                let n_pos = (nx as usize, ny as usize, nz as usize);
                let neighbor_light = &lfs[n_pos];

                // Skip if neighbor has no light
                if neighbor_light.lux <= 0.001 {
                    continue;
                }

                // For X-axis walls, prefer light from left/right
                // For Y-axis walls, prefer light from top/bottom
                let weight = match wall_orientation {
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

                // Apply light with the appropriate weight
                total_lux += neighbor_light.lux * weight * 0.3; // 30% of neighbor's light
                avg_color.0 += neighbor_light.color.0 * weight;
                avg_color.1 += neighbor_light.color.1 * weight;
                avg_color.2 += neighbor_light.color.2 * weight;
                count += weight as usize;
            }

            // Only update if we found lit neighbors
            if count > 0 {
                // Calculate average color
                avg_color.0 /= count as f32;
                avg_color.1 /= count as f32;
                avg_color.2 /= count as f32;

                // Update the light field for this wall
                lfs[(i, j, k)].lux = total_lux;
                lfs[(i, j, k)].color = avg_color;
                walls_lit += 1;
            }
        }
    }

    info!(
        "Wall ambient light pass: {} walls lit in {:?}",
        walls_lit,
        wall_light_start.elapsed()
    );
}

/// Pre-computes static shadow and propagation data for the lighting system.
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

        // Also identify potential continuation points (tiles with see_through=false neighbors)
        let mut has_blocked_neighbor = false;

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
            if nx < 0
                || ny < 0
                || nz < 0
                || nx >= bf.map_size.0 as i64
                || ny >= bf.map_size.1 as i64
                || nz >= bf.map_size.2 as i64
            {
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

            // Check if this might be a continuation point
            // (if we have a neighbor that blocks light)
            if !can_propagate {
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

        // Mark as continuation point if it's at the boundary of blocked/unblocked
        if has_blocked_neighbor
            && (data.propagation_dirs.north
                || data.propagation_dirs.east
                || data.propagation_dirs.south
                || data.propagation_dirs.west)
        {
            data.is_continuation_point = true;
        }
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
    // Queue for BFS processing with (position, source_id, remaining_distance, current_lux, color)
    let mut queue = VecDeque::new();
    let mut visited_by_source = HashMap::new(); // Maps source_id -> set of visited positions

    // Initialize the queue with all light sources
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

            // Add to queue: (position, source_id, remaining_distance, current_lux, color)
            queue.push_back((pos, source_id, 30.0, lux, color));

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

    // BFS processing - extend light waves in all directions
    while let Some((pos, source_id, remaining_distance, current_lux, color)) = queue.pop_front() {
        // Skip if we've reached the distance limit or light is too dim
        if remaining_distance <= 0.0 || current_lux < 0.001 {
            continue;
        }

        let pos_idx = pos.ndidx();

        // FIX: Store propagation directions in local variables to avoid borrowing issues
        // Get the prebaked data in a scope-limited way to just extract what we need
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
            if nx < 0
                || ny < 0
                || nz < 0
                || nx >= bf.map_size.0 as i64
                || ny >= bf.map_size.1 as i64
                || nz >= bf.map_size.2 as i64
            {
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

            // FIX: Split the immutable and mutable accesses to prebaked
            // First get the properties we need for decisions with an immutable borrow
            let (is_continuation_point, transmissivity, see_through) = {
                let neighbor_prebaked = &prebaked[neighbor_idx];
                let collision = &bf.collision_field[neighbor_idx];
                (
                    neighbor_prebaked.is_continuation_point,
                    neighbor_prebaked.light_info.transmissivity,
                    collision.see_through,
                )
            };

            // Now do the mutable operations based on the decisions
            if is_continuation_point && !see_through {
                // Mutable borrow for adding pending propagation
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

            // Calculate diminished light level
            let falloff = 0.75 * transmissivity;
            let new_lux = current_lux * falloff;

            // Mutable borrow for adding/updating contribution
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

            // Add the new light contribution (may receive from multiple paths)
            contribution.lux += new_lux;
            // Keep the maximum remaining distance
            contribution.remaining_distance = contribution
                .remaining_distance
                .max(remaining_distance - 1.0);

            // Add neighbor to queue for next wave propagation
            queue.push_back((
                neighbor_pos,
                source_id,
                remaining_distance - 1.0,
                new_lux,
                color,
            ));

            propagated_light_count += 1;
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
