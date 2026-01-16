use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use ndarray::Array3;
use unboard_core::types::fielddata::CollisionFieldData;
use unspatial_core::boardposition::BoardPosition;
use unthermal_core::resources::TemperatureDiffusionConfig;

/// Calculate connectivity score for a tile at the given position
/// Lower scores = higher processing frequency for temperature diffusion
pub fn calculate_connectivity_score(
    pos: BoardPosition,
    map_size: (usize, usize, usize),
    collision_field: &Array3<CollisionFieldData>,
    config: &TemperatureDiffusionConfig,
) -> u8 {
    // Check bounds
    if pos.x < 0
        || pos.y < 0
        || pos.z < 0
        || pos.x >= map_size.0 as i64
        || pos.y >= map_size.1 as i64
        || pos.z >= map_size.2 as i64
    {
        return config.max_score; // Out of bounds tiles get minimal processing
    }

    let ndidx = pos.ndidx();
    let collision_data = &collision_field[ndidx];

    // Special case: stairs always get highest priority
    if collision_data.stair_offset != 0 {
        return config.stair_score;
    }

    // Special case: closed doors get minimal processing
    if collision_data.is_dynamic {
        return config.door_score;
    }

    if !collision_data.see_through {
        return config.max_score;
    }

    // Count passable 4-way neighbors
    let neighbors = [pos.left(), pos.right(), pos.top(), pos.bottom()];
    let mut passable_neighbors = 0u8;
    let mut second_degree_neighbors = 0u8;

    for neighbor in neighbors.iter() {
        if is_position_passable(neighbor.clone(), map_size, collision_field) {
            passable_neighbors += 1;

            // Count second-degree neighbors (neighbors of neighbors)
            let second_neighbors = [
                neighbor.left(),
                neighbor.right(),
                neighbor.top(),
                neighbor.bottom(),
            ];
            for second_neighbor in second_neighbors.iter() {
                if is_position_passable(second_neighbor.clone(), map_size, collision_field) {
                    second_degree_neighbors += 1;
                }
            }
        }
    }

    // Calculate total connectivity
    passable_neighbors + second_degree_neighbors
}

/// Check if a position is passable (for connectivity calculations)
fn is_position_passable(
    pos: BoardPosition,
    map_size: (usize, usize, usize),
    collision_field: &Array3<CollisionFieldData>,
) -> bool {
    if pos.x < 0
        || pos.y < 0
        || pos.z < 0
        || pos.x >= map_size.0 as i64
        || pos.y >= map_size.1 as i64
        || pos.z >= map_size.2 as i64
    {
        return false;
    }

    let collision_data = &collision_field[pos.ndidx()];
    collision_data.player_free || collision_data.see_through
}

pub fn precompute_connectivity_scores(
    map_size: (usize, usize, usize),
    collision_field: &Array3<CollisionFieldData>,
    connectivity_scores: &mut Array3<u8>,
    config: &TemperatureDiffusionConfig,
) {
    let mut score_distribution = HashMap::new();
    let mut total_tiles = 0;

    for x in 0..map_size.0 {
        for y in 0..map_size.1 {
            for z in 0..map_size.2 {
                let pos = BoardPosition {
                    x: x as i64,
                    y: y as i64,
                    z: z as i64,
                };
                let score = calculate_connectivity_score(pos, map_size, collision_field, config);
                connectivity_scores[(x, y, z)] = score;

                // Track score distribution for logging
                *score_distribution.entry(score).or_insert(0) += 1;
                total_tiles += 1;
            }
        }
    }

    // Log score distribution for debugging
    info!("Connectivity score distribution for {} tiles:", total_tiles);
    for (score, count) in score_distribution.iter() {
        let percentage = (*count as f32 / total_tiles as f32) * 100.0;
        info!(
            "  Score {}: {} tiles ({:.1}% - {:.1}% processing chance)",
            score,
            count,
            percentage,
            100.0 / (*score as f32)
        );
    }
}
