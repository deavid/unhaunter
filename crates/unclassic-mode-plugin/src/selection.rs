use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use bevy_platform::collections::HashSet;
use bevy_platform::time::Instant;
use rand::Rng;
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use unboard_core::resources::board_topology::BoardTopology;
use unfoundation_core::random_seed;
use unghost_core::components::ghost_influence::InfluenceType;
use unspatial_core::position::Position;

/// Represents one complete ghost setup configuration (spawn + influences)
#[derive(Clone, Debug)]
pub(crate) struct GhostSetupCandidate {
    /// Selected ghost spawn position
    pub spawn_point: Position,
    /// Selected entities and their assigned influence types
    pub influence_assignments: Vec<(Entity, InfluenceType)>,
    /// Score representing the quality of this distribution (higher is better)
    pub score: f32,
}

pub(crate) fn select_ghost_spawn_point(
    potential_spawns: &[Position],
    rng: &mut impl Rng,
) -> Option<Position> {
    potential_spawns.choose(rng).copied()
}

pub(crate) fn select_influence_objects(
    objects_by_floor: &HashMap<i64, Vec<Entity>>,
    board_topology: &BoardTopology,
    rng: &mut impl Rng,
) -> Vec<(Entity, InfluenceType)> {
    let mut selected_objects: Vec<(Entity, InfluenceType)> = Vec::new();
    let mut objects_by_floor_copy: HashMap<i64, Vec<Entity>> = objects_by_floor.clone();
    let mut floors_with_requirements = HashSet::new();
    let mut unrestricted_floors: Vec<i64> = Vec::new();
    let mut total_attractive = 0;
    let mut total_repulsive = 0;

    const MIN_ATTRACTIVE: usize = 2;
    const MIN_REPULSIVE: usize = 1;

    for (&floor_z, floor_objects) in &mut objects_by_floor_copy {
        if floor_z < 0 || floor_z >= board_topology.map_size.2 as i64 {
            continue;
        }
        let z_index = floor_z as usize;
        if let Some(&tiled_floor_num) = board_topology.z_floor_map.get(&z_index) {
            if let Some(&attract_count) = board_topology
                .floor_mapping
                .ghost_attracting_objects
                .get(&tiled_floor_num)
                && attract_count > 0
            {
                floors_with_requirements.insert(floor_z);
                if floor_objects.len() >= attract_count as usize {
                    let mut floor_objects_clone = floor_objects.clone();
                    floor_objects_clone.shuffle(rng);
                    let take_count = attract_count as usize;
                    for entity in floor_objects_clone.iter().take(take_count) {
                        selected_objects.push((*entity, InfluenceType::Attractive));
                        total_attractive += 1;
                    }
                    for entity in selected_objects
                        .iter()
                        .filter(|(_, t)| *t == InfluenceType::Attractive)
                        .map(|(e, _)| e)
                    {
                        if let Some(pos) = floor_objects.iter().position(|&e| e == *entity) {
                            floor_objects.swap_remove(pos);
                        }
                    }
                }
            }
            if let Some(&repel_count) = board_topology
                .floor_mapping
                .ghost_repelling_objects
                .get(&tiled_floor_num)
                && repel_count > 0
            {
                floors_with_requirements.insert(floor_z);
                if floor_objects.len() >= repel_count as usize {
                    let mut floor_objects_clone = floor_objects.clone();
                    floor_objects_clone.shuffle(rng);
                    let take_count = repel_count as usize;
                    for entity in floor_objects_clone.iter().take(take_count) {
                        selected_objects.push((*entity, InfluenceType::Repulsive));
                        total_repulsive += 1;
                    }
                    for entity in selected_objects
                        .iter()
                        .filter(|(_, t)| *t == InfluenceType::Repulsive)
                        .map(|(e, _)| e)
                    {
                        if let Some(pos) = floor_objects.iter().position(|&e| e == *entity) {
                            floor_objects.swap_remove(pos);
                        }
                    }
                }
            }
        }
    }

    for (&floor_z, _) in &objects_by_floor_copy {
        if !floors_with_requirements.contains(&floor_z) {
            unrestricted_floors.push(floor_z);
        }
    }

    if total_repulsive < MIN_REPULSIVE {
        let needed = MIN_REPULSIVE - total_repulsive;
        for floor_z in &unrestricted_floors {
            if let Some(floor_objects) = objects_by_floor_copy.get_mut(floor_z)
                && !floor_objects.is_empty()
            {
                floor_objects.shuffle(rng);
                let take_count = needed.min(floor_objects.len());
                for entity in floor_objects.iter().take(take_count) {
                    selected_objects.push((*entity, InfluenceType::Repulsive));
                    total_repulsive += 1;
                }
                floor_objects.drain(0..take_count);
                if total_repulsive >= MIN_REPULSIVE {
                    break;
                }
            }
        }
    }

    if total_attractive < MIN_ATTRACTIVE {
        let needed = MIN_ATTRACTIVE - total_attractive;
        for floor_z in &unrestricted_floors {
            if let Some(floor_objects) = objects_by_floor_copy.get_mut(floor_z)
                && !floor_objects.is_empty()
            {
                floor_objects.shuffle(rng);
                let take_count = needed.min(floor_objects.len());
                for entity in floor_objects.iter().take(take_count) {
                    selected_objects.push((*entity, InfluenceType::Attractive));
                    total_attractive += 1;
                }
                floor_objects.drain(0..take_count);
                if total_attractive >= MIN_ATTRACTIVE {
                    break;
                }
            }
        }
    }

    selected_objects
}

fn score_ghost_setup(
    ghost_spawn: &Position,
    influence_objects: &[(Entity, InfluenceType, Position)],
    player_spawns: &[Position],
) -> f32 {
    let mut distances = Vec::new();
    let player_pos = if !player_spawns.is_empty() {
        let mut rng = random_seed::rng();
        let index = rng.random_range(0..player_spawns.len());
        Some(&player_spawns[index])
    } else {
        None
    };

    for (_, _, obj_pos) in influence_objects {
        distances.push(ghost_spawn.distance(obj_pos));
    }
    if let Some(pos) = player_pos {
        distances.push(ghost_spawn.distance(pos));
    }
    for (i, (_, _, pos1)) in influence_objects.iter().enumerate() {
        for (_, _, pos2) in influence_objects.iter().skip(i + 1) {
            distances.push(pos1.distance(pos2));
        }
    }

    if distances.is_empty() {
        return 0.0;
    }

    let sum_logs: f32 = distances
        .iter()
        .map(|&d| if d > 0.0 { d.ln() } else { -10.0 })
        .sum();
    let n = distances.len() as f32;
    (sum_logs / n).exp()
}

pub(crate) fn generate_scored_ghost_setup(
    ghost_spawn_points: &[Position],
    objects_by_floor_with_positions: &HashMap<i64, Vec<(Entity, Position)>>,
    player_spawn_points: &[Position],
    board_toplogy: &BoardTopology,
    simulation_count: usize,
) -> (Position, Vec<(Entity, InfluenceType)>) {
    let start_time = Instant::now();
    let mut objects_by_floor: HashMap<i64, Vec<Entity>> = HashMap::new();
    let mut entity_positions: HashMap<Entity, Position> = HashMap::new();

    for (&floor_z, objects_with_pos) in objects_by_floor_with_positions {
        let entities: Vec<Entity> = objects_with_pos.iter().map(|(e, _)| *e).collect();
        objects_by_floor.insert(floor_z, entities);
        for &(entity, position) in objects_with_pos {
            entity_positions.insert(entity, position);
        }
    }

    let mut candidates: Vec<GhostSetupCandidate> = Vec::with_capacity(simulation_count);
    let mut rng = random_seed::rng();

    for _ in 0..simulation_count {
        let spawn_point = select_ghost_spawn_point(ghost_spawn_points, &mut rng)
            .unwrap_or(Position::new_i64(0, 0, 0));
        let influence_assignments =
            select_influence_objects(&objects_by_floor, board_toplogy, &mut rng);
        let mut influence_with_pos = Vec::new();
        for (entity, influence_type) in &influence_assignments {
            if let Some(position) = entity_positions.get(entity) {
                influence_with_pos.push((*entity, *influence_type, *position));
            }
        }
        let score = score_ghost_setup(&spawn_point, &influence_with_pos, player_spawn_points);
        let score = (score / 16.0).powi(5);
        candidates.push(GhostSetupCandidate {
            spawn_point,
            influence_assignments,
            score,
        });
    }

    let total_score: f32 = candidates.iter().map(|c| c.score.max(0.001)).sum();
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
        candidates.remove(rng.random_range(0..candidates.len()))
    };

    let elapsed = start_time.elapsed();
    debug!(
        "Ghost setup simulation completed: {} simulations in {:.2?}. Selected setup score: {:.2}",
        simulation_count, elapsed, chosen_setup.score
    );

    (chosen_setup.spawn_point, chosen_setup.influence_assignments)
}

pub(crate) fn select_influence_objects_with_simulation(
    objects_by_floor_with_positions: &HashMap<i64, Vec<(Entity, Position)>>,
    ghost_spawn_points: &[Position],
    player_spawn_points: &[Position],
    board_toplogy: &BoardTopology,
) -> (Position, Vec<(Entity, InfluenceType)>) {
    generate_scored_ghost_setup(
        ghost_spawn_points,
        objects_by_floor_with_positions,
        player_spawn_points,
        board_toplogy,
        64,
    )
}
