use crate::cached_board_pos::CachedBoardPos;
use bevy::{prelude::*, utils::Instant};
use fastapprox::faster;
use ndarray::Array3;
use std::{collections::VecDeque, time::Duration};
use uncore::{
    behavior::{Behavior, Orientation},
    components::board::{boardposition::BoardPosition, position::Position},
    resources::board_data::BoardData,
    types::board::{
        fielddata::LightFieldData,
        prebaked_lighting_data::{LightInfo, PrebakedLightingData},
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

    // Collect dynamic light sources that might have changed since prebake
    // (like switched-on lights or newly opened doors)
    let mut dynamic_light_count = 0;
    for (pos, behavior) in qt.iter() {
        let board_pos = pos.to_board_position();
        let idx = board_pos.ndidx();

        // Add any dynamic light sources to our light field
        let lux = behavior.p.light.emmisivity_lumens();
        if lux > 0.0 {
            dynamic_light_count += 1;
            lfs[idx] = LightFieldData {
                lux,
                color: behavior.p.light.color(),
                transmissivity: behavior.p.light.transmissivity_factor(),
                additional: behavior.p.light.additional_data(),
            };
        }
    }
    info!("Dynamic light sources found: {}", dynamic_light_count);

    // Create a visited set to track which tiles have been processed
    // We'll use this to prevent duplicate work and handle wave collisions
    let mut visited = Array3::from_elem(bf.map_size, false);

    // Queue for BFS processing
    let mut queue = VecDeque::new();

    // Count seeds from prebaked data
    let mut seed_count = 0;

    // Seed the queue with all light sources (based on prebaked data)
    for ((i, j, k), prebaked_data) in bf.prebaked_lighting.indexed_iter() {
        if prebaked_data.light_info.is_source {
            seed_count += 1;
            let pos = BoardPosition {
                x: i as i64,
                y: j as i64,
                z: k as i64,
            };

            // Add the light source to the BFS queue with distance and lux
            queue.push_back((pos.clone(), 30.0, prebaked_data.light_info.lux));

            // Update the light field with prebaked source data
            lfs[pos.ndidx()] = LightFieldData {
                lux: prebaked_data.light_info.lux,
                color: prebaked_data.light_info.color,
                transmissivity: prebaked_data.light_info.transmissivity,
                ..Default::default()
            };

            // Mark as visited
            visited[pos.ndidx()] = true;
        }
    }
    info!("Light sources seeded from prebaked data: {}", seed_count);

    if seed_count == 0 {
        warn!("No light sources found in prebaked data! Map will be dark.");
    }

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

    // Track propagation statistics
    let mut propagated_light_count = 0;
    let mut max_distance = ordered_float::OrderedFloat(0.0);
    let mut total_tiles_lit = 0;

    // BFS processing - extend light waves in all directions
    while let Some((pos, remaining_distance, current_lux)) = queue.pop_front() {
        // Skip if we've reached the distance limit or light is too dim
        if remaining_distance <= 0.0 || current_lux < 0.001 {
            continue;
        }

        let pos_idx = pos.ndidx();

        // Track maximum propagation distance
        let distance_traveled = 30.0 - remaining_distance;
        max_distance = max_distance.max(ordered_float::OrderedFloat(distance_traveled));

        // Get the prebaked data for this position
        let prebaked_data = &bf.prebaked_lighting[pos_idx];

        // Log this position being processed
        if propagated_light_count == 0 || propagated_light_count < 10 {
            info!(
                "Processing light at {:?}, remaining_distance={}, current_lux={}",
                pos, remaining_distance, current_lux
            );
        }

        // Propagate in each direction
        for (dir_i, &(dx, dy, dz)) in directions.iter().enumerate() {
            let dir_name = match dir_i {
                0 => "North",
                1 => "East",
                2 => "South",
                3 => "West",
                _ => "Unknown",
            };

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
                if propagated_light_count < 10 {
                    info!("  -> {} blocked by collision", dir_name);
                }
                continue;
            }

            // Check if the current position allows propagation in this direction
            let can_propagate = match dir_i {
                0 => prebaked_data.propagation_dirs.north,
                1 => prebaked_data.propagation_dirs.east,
                2 => prebaked_data.propagation_dirs.south,
                3 => prebaked_data.propagation_dirs.west,
                _ => false,
            };

            if !can_propagate {
                if propagated_light_count < 10 {
                    info!("  -> {} blocked by propagation direction", dir_name);
                }
                continue;
            }

            // Check if already visited
            if visited[neighbor_idx] {
                if propagated_light_count < 10 {
                    info!("  -> {} already visited", dir_name);
                }
                continue; // Wave collision - both waves stop
            }

            // Get the prebaked data for the neighbor
            let neighbor_prebaked = &bf.prebaked_lighting[neighbor_idx];

            // If the neighbor is a light source too, handle differently
            if neighbor_prebaked.light_info.is_source {
                if propagated_light_count < 10 {
                    info!("  -> {} is a light source", dir_name);
                }
                continue;
            }

            // Calculate diminished light level (using neighbor's transmissivity from prebaked data)
            let transmissivity = neighbor_prebaked.light_info.transmissivity;
            let falloff = 0.75 * transmissivity;
            let new_lux = current_lux * falloff;

            // Update light field for neighbor
            lfs[neighbor_idx].lux += new_lux;

            // Use the color from the light source that reached this tile
            if lfs[neighbor_idx].lux > 0.0 {
                lfs[neighbor_idx].color = prebaked_data.light_info.color;
            }

            // Add neighbor to queue for next wave propagation
            queue.push_back((neighbor_pos, remaining_distance - 1.0, new_lux));

            if propagated_light_count < 10 {
                info!(
                    "  -> {} propagated: lux={}, new_distance={}",
                    dir_name,
                    new_lux,
                    remaining_distance - 1.0
                );
            }

            propagated_light_count += 1;
            total_tiles_lit += 1;

            // Mark as visited
            visited[neighbor_idx] = true;
        }
    }

    info!(
        "BFS propagation stats: {} light propagations, {} total tiles lit, max distance: {}",
        propagated_light_count, total_tiles_lit, max_distance
    );

    // Count how many tiles have light
    let tiles_with_light = lfs.iter().filter(|x| x.lux > 0.0).count();
    let total_tiles = bf.map_size.0 * bf.map_size.1 * bf.map_size.2;
    let avg_lux = lfs.iter().map(|x| x.lux).sum::<f32>() / total_tiles as f32;
    let max_lux = lfs.iter().map(|x| x.lux).fold(0.0, f32::max);

    info!(
        "Light field after BFS: {}/{} tiles lit ({:.2}%), avg lux: {:.6}, max lux: {:.6}",
        tiles_with_light,
        total_tiles,
        (tiles_with_light as f32 / total_tiles as f32) * 100.0,
        avg_lux,
        max_lux
    );

    // ADDED STEP: Apply ambient light to walls based on neighbors
    // This will make walls visible with soft lighting from adjacent tiles
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
                lux,
                color: behavior.p.light.color(),
                transmissivity: behavior.p.light.transmissivity_factor(),
            };
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

    // Store the prebaked data in BoardData
    bf.prebaked_lighting = prebaked;

    info!(
        "Prebaked lighting field computed in: {:?}",
        build_start_time.elapsed()
    );
}
