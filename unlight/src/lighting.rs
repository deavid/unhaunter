use crate::utils::{
    apply_prebaked_contributions, identify_active_light_sources, is_in_bounds,
    propagate_from_wave_edges, update_exposure_and_stats,
};
use bevy::{prelude::*, utils::Instant};
use ndarray::Array3;
use uncore::{
    behavior::{Behavior, Orientation},
    components::board::position::Position,
    resources::board_data::BoardData,
    types::board::fielddata::LightFieldData,
};

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
pub fn rebuild_lighting_field(
    bf: &mut BoardData,
    qt: &Query<(&Position, &Behavior)>,
    avg_time: &mut Local<(f32, f32)>,
) {
    info!("Starting rebuild_lighting_field using prebaked data");
    let build_start_time = Instant::now();

    // Create a new light field with default values
    let mut lfs = Array3::from_elem(bf.map_size, LightFieldData::default());

    // Identify active light sources
    let active_source_ids = identify_active_light_sources(bf, qt);

    // Apply prebaked contributions from active sources
    let _initial_tiles_lit = apply_prebaked_contributions(&active_source_ids, bf, &mut lfs);

    let _dynamic_propagation_count = propagate_from_wave_edges(bf, &mut lfs, &active_source_ids);

    // info!(
    //     "Dynamic BFS propagation: {} additional light propagations, {} initial tiles lit in {:?}",
    //     dynamic_propagation_count,
    //     initial_tiles_lit,
    //     time_dynamic_propagation.elapsed()
    // );

    // Apply ambient light to walls
    apply_ambient_light_to_walls(bf, &mut lfs);

    // Calculate exposure and update board data
    update_exposure_and_stats(bf, &lfs);

    let total_time = build_start_time.elapsed().as_secs_f32();
    let tot_cnt = 4.0;
    avg_time.0 = (avg_time.0 * avg_time.1 + total_time * tot_cnt) / (avg_time.1 + tot_cnt);
    avg_time.1 += 1.0;
    info!(
        "BFS light propagation completed in: {:?} (mean {:.2}ms)",
        build_start_time.elapsed(),
        avg_time.0 * 1000.0
    );
}

// Applies ambient light to walls based on neighboring lit tiles
fn apply_ambient_light_to_walls(bf: &BoardData, lfs: &mut Array3<LightFieldData>) {
    let wall_light_start = Instant::now();
    let mut walls_lit = 0;

    // // Define directions for 4-way connectivity (plus weight)
    let directions = [
        (0, 1, 0, 0.01),
        (1, -1, 0, 0.1),
        (1, 0, 0, 1.0),
        (0, -1, 0, 1.0),
        (-1, 0, 0, 0.01),
    ];

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
            walls_lit += 1;
        }
    }

    info!(
        "Wall ambient light pass: {} walls lit in {:?}",
        walls_lit,
        wall_light_start.elapsed()
    );
}
