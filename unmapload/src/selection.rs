use bevy::prelude::*;
use bevy::utils::{HashMap, Instant};
use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use uncore::components::board::position::Position;
use uncore::components::ghost_influence::InfluenceType;
use uncore::random_seed;
use uncore::resources::board_data::BoardData;

/// Represents one complete ghost setup configuration (spawn + influences)
#[derive(Clone, Debug)]
pub struct GhostSetupCandidate {
    /// Selected ghost spawn position
    pub spawn_point: Position,
    /// Selected entities and their assigned influence types
    pub influence_assignments: Vec<(Entity, InfluenceType)>,
    /// Score representing the quality of this distribution (higher is better)
    pub score: f32,
}

/// Selects a ghost spawn point from a list of potential spawn positions.
///
/// # Arguments
/// * `potential_spawns` - List of potential ghost spawn positions
/// * `rng` - Random number generator
///
/// # Returns
/// * `Option<Position>` - The selected ghost spawn position, or None if no positions are available
pub fn select_ghost_spawn_point(
    potential_spawns: &[Position],
    rng: &mut impl Rng,
) -> Option<Position> {
    use rand::prelude::IndexedRandom;
    // Simple random selection - choose one point from the list if available
    potential_spawns.choose(rng).copied()
}

/// Selects objects to be assigned ghost influence properties, respecting floor requirements.
///
/// This function handles both floor-specific requirements and global minimums:
/// - If a floor has specific requirements, it will select the exact number of objects requested
/// - If there aren't enough specified requirements, it will add objects to meet global minimums
///
/// # Arguments
/// * `objects_by_floor` - Map of movable objects grouped by floor Z-coordinate
/// * `board_data` - Board data resource containing floor requirements
/// * `rng` - Random number generator
///
/// # Returns
/// * `Vec<(Entity, InfluenceType)>` - List of selected entities and their assigned influence types
pub fn select_influence_objects(
    objects_by_floor: &HashMap<i64, Vec<Entity>>,
    board_data: &BoardData,
    rng: &mut impl Rng,
) -> Vec<(Entity, InfluenceType)> {
    // Track selected objects and influence types to be assigned
    let mut selected_objects: Vec<(Entity, InfluenceType)> = Vec::new();

    // Make a mutable copy of the objects map to work with
    let mut objects_by_floor_copy: HashMap<i64, Vec<Entity>> = objects_by_floor.clone();

    // Track floors with specific requirements
    let mut floors_with_requirements = HashSet::new();

    // Track floors without specific requirements to use them for global minimums
    let mut unrestricted_floors: Vec<i64> = Vec::new();

    // Track running count to ensure we meet global minimums
    let mut total_attractive = 0;
    let mut total_repulsive = 0;

    // Minimum required counts
    const MIN_ATTRACTIVE: usize = 2;
    const MIN_REPULSIVE: usize = 1;

    // First, handle floors with specific requirements
    for (&floor_z, floor_objects) in &mut objects_by_floor_copy {
        // Convert from i64 to usize for z_floor_map lookup
        if floor_z < 0 || floor_z >= board_data.map_size.2 as i64 {
            // Skip floors outside the valid range
            continue;
        }

        let z_index = floor_z as usize;

        // Look up the original floor number (from the TMX file)
        if let Some(&tiled_floor_num) = board_data.z_floor_map.get(&z_index) {
            // Check if this floor has specific requirements for ghost attracting objects
            if let Some(&attract_count) = board_data
                .floor_mapping
                .ghost_attracting_objects
                .get(&tiled_floor_num)
            {
                if attract_count > 0 {
                    // info!(
                    //     "Floor {floor_z} (Tiled floor {tiled_floor_num}) requires {attract_count} attractive objects"
                    // );
                    floors_with_requirements.insert(floor_z);

                    // Ensure we have enough objects for this floor
                    if floor_objects.len() < attract_count as usize {
                        // warn!(
                        //     "Floor {floor_z} requires {attract_count} attractive objects but only has {} movable objects",
                        //     floor_objects.len()
                        // );
                        continue;
                    }

                    // Shuffle objects for this floor to randomize selection
                    let mut floor_objects_clone = floor_objects.clone();
                    floor_objects_clone.shuffle(rng);

                    // Take what we need from this floor
                    let take_count = attract_count as usize;
                    for entity in floor_objects_clone.iter().take(take_count) {
                        selected_objects.push((*entity, InfluenceType::Attractive));
                        total_attractive += 1;
                    }

                    // Remove selected objects from the available pool
                    for entity in selected_objects
                        .iter()
                        .filter(|(_, influence_type)| *influence_type == InfluenceType::Attractive)
                        .map(|(entity, _)| entity)
                    {
                        if let Some(pos) = floor_objects.iter().position(|&e| e == *entity) {
                            floor_objects.swap_remove(pos);
                        }
                    }
                }
            }

            // Check if this floor has specific requirements for ghost repelling objects
            if let Some(&repel_count) = board_data
                .floor_mapping
                .ghost_repelling_objects
                .get(&tiled_floor_num)
            {
                if repel_count > 0 {
                    // info!(
                    //     "Floor {floor_z} (Tiled floor {tiled_floor_num}) requires {repel_count} repulsive objects"
                    // );
                    floors_with_requirements.insert(floor_z);

                    // Ensure we have enough objects for this floor
                    if floor_objects.len() < repel_count as usize {
                        // warn!(
                        //     "Floor {floor_z} requires {repel_count} repulsive objects but only has {} movable objects",
                        //     floor_objects.len()
                        // );
                        continue;
                    }

                    // Shuffle objects for this floor to randomize selection
                    let mut floor_objects_clone = floor_objects.clone();
                    floor_objects_clone.shuffle(rng);

                    // Take what we need from this floor
                    let take_count = repel_count as usize;
                    for entity in floor_objects_clone.iter().take(take_count) {
                        selected_objects.push((*entity, InfluenceType::Repulsive));
                        total_repulsive += 1;
                    }

                    // Remove selected objects from the available pool
                    for entity in selected_objects
                        .iter()
                        .filter(|(_, influence_type)| *influence_type == InfluenceType::Repulsive)
                        .map(|(entity, _)| entity)
                    {
                        if let Some(pos) = floor_objects.iter().position(|&e| e == *entity) {
                            floor_objects.swap_remove(pos);
                        }
                    }
                }
            }
        }
    }

    // Identify floors without specific requirements
    for (&floor_z, _) in &objects_by_floor_copy {
        if !floors_with_requirements.contains(&floor_z) {
            unrestricted_floors.push(floor_z);
        }
    }

    // Ensure we meet the global minimum requirements for repulsive objects
    if total_repulsive < MIN_REPULSIVE {
        let needed = MIN_REPULSIVE - total_repulsive;
        //cinfo!("Need to add {needed} more repulsive objects to meet global minimum");

        // Try to find objects from unrestricted floors
        for floor_z in &unrestricted_floors {
            if let Some(floor_objects) = objects_by_floor_copy.get_mut(floor_z) {
                if !floor_objects.is_empty() {
                    // Shuffle objects for this floor to randomize selection
                    floor_objects.shuffle(rng);

                    // Take what we need from this floor
                    let take_count = needed.min(floor_objects.len());
                    for entity in floor_objects.iter().take(take_count) {
                        selected_objects.push((*entity, InfluenceType::Repulsive));
                        total_repulsive += 1;
                    }

                    // Remove selected objects
                    floor_objects.drain(0..take_count);

                    // Break if we've met the requirement
                    if total_repulsive >= MIN_REPULSIVE {
                        break;
                    }
                }
            }
        }
    }

    // Ensure we meet the global minimum requirements for attractive objects
    if total_attractive < MIN_ATTRACTIVE {
        let needed = MIN_ATTRACTIVE - total_attractive;
        // info!("Need to add {needed} more attractive objects to meet global minimum");

        // Try to find objects from unrestricted floors
        for floor_z in &unrestricted_floors {
            if let Some(floor_objects) = objects_by_floor_copy.get_mut(floor_z) {
                if !floor_objects.is_empty() {
                    // Shuffle objects for this floor to randomize selection
                    floor_objects.shuffle(rng);

                    // Take what we need from this floor
                    let take_count = needed.min(floor_objects.len());
                    for entity in floor_objects.iter().take(take_count) {
                        selected_objects.push((*entity, InfluenceType::Attractive));
                        total_attractive += 1;
                    }

                    // Remove selected objects
                    floor_objects.drain(0..take_count);

                    // Break if we've met the requirement
                    if total_attractive >= MIN_ATTRACTIVE {
                        break;
                    }
                }
            }
        }
    }

    // Log the final distribution
    // info!(
    //     "Ghost influence selection complete: {} attractive and {} repulsive objects",
    //     total_attractive, total_repulsive
    // );

    selected_objects
}

/// Calculates a score based on spatial distribution of selected elements
///
/// This function calculates the "product mean" of distances between all relevant
/// elements (ghost spawn, player spawns, and influence objects) using the logarithmic
/// approach to avoid overflow issues. Higher scores indicate better spatial distribution.
///
/// # Arguments
/// * `ghost_spawn` - Position of the ghost spawn point
/// * `influence_objects` - List of selected influence objects with their positions
/// * `player_spawns` - List of player spawn positions
///
/// # Returns
/// * Score based on the distribution of objects (higher is better)
fn score_ghost_setup(
    ghost_spawn: &Position,
    influence_objects: &[(Entity, InfluenceType, Position)],
    player_spawns: &[Position],
) -> f32 {
    let mut distances = Vec::new();

    // Calculate ghost spawn to influence object distances
    for (_, _, obj_pos) in influence_objects {
        let dist = ghost_spawn.distance(obj_pos);
        distances.push(dist);
    }

    // Calculate ghost spawn to player spawn distances
    for player_pos in player_spawns {
        let dist = ghost_spawn.distance(player_pos);
        distances.push(dist);
    }

    // Calculate influence object to influence object distances
    for (i, (_, _, pos1)) in influence_objects.iter().enumerate() {
        for (_, _, pos2) in influence_objects.iter().skip(i + 1) {
            let dist = pos1.distance(pos2);
            distances.push(dist);
        }
    }

    // Calculate influence object to player spawn distances
    for (_, _, obj_pos) in influence_objects {
        for player_pos in player_spawns {
            let dist = obj_pos.distance(player_pos);
            distances.push(dist);
        }
    }

    // If no distances were calculated, return a default score of 0
    if distances.is_empty() {
        return 0.0;
    }

    // Using logarithmic approach to avoid overflow
    // Sum of logs instead of product
    let sum_logs: f32 = distances
        .iter()
        .map(|&d| if d > 0.0 { d.ln() } else { -10.0 }) // Handle zero distances safely
        .sum();

    // Divide by n and exponentiate to get geometric mean
    let n = distances.len() as f32;
    (sum_logs / n).exp()
}

/// Generates multiple ghost setup candidates and selects one using weighted random selection.
///
/// This function:
/// 1. Runs multiple simulations of ghost setups with different random selections
/// 2. Scores each setup based on spatial distribution
/// 3. Selects one using weighted random selection based on scores
///
/// # Arguments
/// * `ghost_spawn_points` - List of potential ghost spawn positions
/// * `objects_by_floor_with_positions` - Map of movable objects with their positions
/// * `player_spawn_points` - List of player spawn positions
/// * `board_data` - Board data resource containing floor requirements
/// * `rng` - Random number generator
/// * `simulation_count` - Number of simulations to run
///
/// # Returns
/// * Selected ghost spawn position and influence object assignments
pub fn generate_scored_ghost_setup(
    ghost_spawn_points: &[Position],
    objects_by_floor_with_positions: &HashMap<i64, Vec<(Entity, Position)>>,
    player_spawn_points: &[Position],
    board_data: &BoardData,
    simulation_count: usize,
) -> (Position, Vec<(Entity, InfluenceType)>) {
    // Start timing the simulation
    let start_time = Instant::now();

    // Convert to the format needed by select_influence_objects
    let mut objects_by_floor: HashMap<i64, Vec<Entity>> = HashMap::new();
    let mut entity_positions: HashMap<Entity, Position> = HashMap::new();

    for (&floor_z, objects_with_pos) in objects_by_floor_with_positions {
        let entities: Vec<Entity> = objects_with_pos.iter().map(|(e, _)| *e).collect();
        objects_by_floor.insert(floor_z, entities);

        for &(entity, position) in objects_with_pos {
            entity_positions.insert(entity, position);
        }
    }

    // Generate multiple candidate setups
    let mut candidates: Vec<GhostSetupCandidate> = Vec::with_capacity(simulation_count);
    let mut rng = random_seed::rng();

    for _ in 0..simulation_count {
        // Create a new RNG for each simulation using a random seed from the main RNG

        // Select a ghost spawn point
        let spawn_point =
            select_ghost_spawn_point(ghost_spawn_points, &mut rng).unwrap_or_else(|| {
                warn!("No ghost spawn points available for simulation!");
                Position::new_i64(0, 0, 0) // Fallback position
            });

        // Select influence objects
        let influence_assignments =
            select_influence_objects(&objects_by_floor, board_data, &mut rng);

        // Prepare data for scoring
        let mut influence_with_pos = Vec::new();
        for (entity, influence_type) in &influence_assignments {
            if let Some(position) = entity_positions.get(entity) {
                influence_with_pos.push((*entity, *influence_type, *position));
            } else {
                warn!("Position not found for entity {:?}", entity);
            }
        }

        // Score this setup
        let score = score_ghost_setup(&spawn_point, &influence_with_pos, player_spawn_points);
        let score = (score / 16.0).powi(5);
        // Store this candidate
        candidates.push(GhostSetupCandidate {
            spawn_point,
            influence_assignments,
            score,
        });
    }

    // Weighted random selection based on scores
    let total_score: f32 = candidates.iter().map(|c| c.score.max(0.001)).sum();

    // Choose a setup using weighted random selection
    let chosen_setup = if total_score > 0.0 {
        let mut choice_value = rng.random::<f32>() * total_score;
        let mut chosen_idx = 0;

        for (i, candidate) in candidates.iter().enumerate() {
            choice_value -= candidate.score.max(0.001);
            if choice_value <= 0.0 {
                chosen_idx = i;
                break;
            }
        }

        candidates.remove(chosen_idx)
    } else {
        // If all scores are 0, pick one randomly
        candidates.remove(rng.random_range(0..candidates.len()))
    };

    // Measure and log elapsed time
    let elapsed = start_time.elapsed();

    // Calculate statistics about the scores
    let avg_score = if !candidates.is_empty() {
        candidates.iter().map(|c| c.score).sum::<f32>() / candidates.len() as f32
    } else {
        0.0
    };

    let min_score = candidates
        .iter()
        .map(|c| c.score)
        .fold(f32::MAX, |a, b| a.min(b));
    let max_score = candidates
        .iter()
        .map(|c| c.score)
        .fold(f32::MIN, |a, b| a.max(b));

    info!(
        "Ghost setup simulation completed: {} simulations in {:.2?}. Selected setup score: {:.2}, avg: {:.2}, min: {:.2}, max: {:.2}",
        simulation_count, elapsed, chosen_setup.score, avg_score, min_score, max_score
    );

    (chosen_setup.spawn_point, chosen_setup.influence_assignments)
}

/// Selects objects to be assigned ghost influence properties using simulation and spatial scoring.
///
/// This function enhances the regular selection process by running multiple simulations and
/// scoring them based on spatial distribution to avoid clustering of objects.
///
/// # Arguments
/// * `objects_by_floor_with_positions` - Map of movable objects with their positions
/// * `ghost_spawn_points` - List of potential ghost spawn positions
/// * `player_spawn_points` - List of player spawn positions
/// * `board_data` - Board data resource containing floor requirements
/// * `rng` - Random number generator
///
/// # Returns
/// * `(Position, Vec<(Entity, InfluenceType)>)` - Selected ghost spawn and influence assignments
pub fn select_influence_objects_with_simulation(
    objects_by_floor_with_positions: &HashMap<i64, Vec<(Entity, Position)>>,
    ghost_spawn_points: &[Position],
    player_spawn_points: &[Position],
    board_data: &BoardData,
) -> (Position, Vec<(Entity, InfluenceType)>) {
    // Run the simulation with 64 candidates
    generate_scored_ghost_setup(
        ghost_spawn_points,
        objects_by_floor_with_positions,
        player_spawn_points,
        board_data,
        64, // Number of simulations
    )
}
