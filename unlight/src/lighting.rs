use crate::{
    cached_board_pos::CachedBoardPos,
    utils::{
        apply_prebaked_contributions, collect_door_states, find_wave_edge_tiles,
        identify_active_light_sources, is_in_bounds, propagate_from_wave_edges,
        update_exposure_and_stats,
    },
};
use bevy::{prelude::*, utils::Instant};
use fastapprox::faster;
use ndarray::Array3;
use std::time::Duration;
use uncore::{
    behavior::{Behavior, Orientation},
    components::board::{boardposition::BoardPosition, position::Position},
    resources::board_data::BoardData,
    types::board::fielddata::LightFieldData,
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
    let active_source_ids = identify_active_light_sources(bf, qt);

    // 2. Apply prebaked contributions from active sources
    let initial_tiles_lit = apply_prebaked_contributions(&active_source_ids, bf, &mut lfs);

    // 3. Handle door states
    let door_states = collect_door_states(qt);

    // 4. Find wave edge tiles that need propagation
    let wave_edges = find_wave_edge_tiles(bf, &active_source_ids, &door_states);

    // 5. Propagate light from wave edges
    let dynamic_propagation_count = propagate_from_wave_edges(bf, &mut lfs, &wave_edges);

    info!(
        "Dynamic BFS propagation: {} additional light propagations, {} initial tiles lit",
        dynamic_propagation_count, initial_tiles_lit
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

    // // Define directions for 4-way connectivity
    // let directions = [(0, 1, 0), (1, 0, 0), (0, -1, 0), (-1, 0, 0)];

    // Only 2 directions to lit up the external walls on their facing side from the camera POV.
    let directions = [(1, 0, 0), (0, -1, 0)];

    // Threshold for considering a tile "dark"
    const DARK_THRESHOLD: f32 = 0.000001;

    let src_lfs = lfs.clone();

    for ((i, j, k), collision) in bf.collision_field.indexed_iter() {
        // Only process dark tiles
        if src_lfs[(i, j, k)].lux > DARK_THRESHOLD {
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
            let neighbor_light = &src_lfs[n_pos];

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
            let lfs_idx = &mut lfs[(i, j, k)];
            lfs_idx.lux = total_lux;
            lfs_idx.color = avg_color;
            walls_lit += 1;
        }
    }

    info!(
        "Wall ambient light pass: {} walls lit in {:?}",
        walls_lit,
        wall_light_start.elapsed()
    );
}
