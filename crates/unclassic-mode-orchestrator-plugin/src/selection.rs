use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use bevy_platform::collections::HashSet;
use bevy_platform::time::Instant;
use rand::Rng;
use rand::RngExt;
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use unboard_core::resources::board_topology::BoardTopology;
use uncommon_app_core::random_seed;
use unghost_core::components::logic::ghost_influence::InfluenceType;
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

const CROSS_FLOOR_DISTANCE_PENALTY: f32 = 20.0;
const DEBUG_SELECT_MAX: bool = false;
const EA_POPULATION_SIZE: usize = 50;
const EA_ELITE_COUNT: usize = 10;
const EA_GENERATIONS: usize = 3;
const EA_MUTATION_BREACH_CHANCE: f32 = 0.5;

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
        unrestricted_floors.shuffle(rng);
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
        unrestricted_floors.shuffle(rng);
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

fn score_distance(a: &Position, b: &Position) -> f32 {
    let mut distance = a.distance(b);
    if a.z.trunc() != b.z.trunc() {
        distance += CROSS_FLOOR_DISTANCE_PENALTY;
    }
    distance
}

fn calculate_set_signature(set: &[Position], origin: &Position) -> [f32; 6] {
    if set.is_empty() {
        return [0.0; 6];
    }

    let mut mean_x = 0.0;
    let mut mean_y = 0.0;
    let mut mean_z = 0.0;

    for p in set {
        mean_x += p.x - origin.x;
        mean_y += p.y - origin.y;
        mean_z += p.z - origin.z;
    }

    let n = set.len() as f32;
    mean_x /= n;
    mean_y /= n;
    mean_z /= n;

    let mut spread_x = 0.0;
    let mut spread_y = 0.0;
    let mut spread_z = 0.0;

    for p in set {
        spread_x += ((p.x - origin.x) - mean_x).abs();
        spread_y += ((p.y - origin.y) - mean_y).abs();
        spread_z += ((p.z - origin.z) - mean_z).abs();
    }

    spread_x /= n;
    spread_y /= n;
    spread_z /= n;

    [mean_x, mean_y, mean_z, spread_x, spread_y, spread_z]
}

fn signature_distance(sig_a: &[f32; 6], sig_b: &[f32; 6]) -> f32 {
    let mut dist_sq = 0.0;
    for i in 0..6 {
        dist_sq += (sig_a[i] - sig_b[i]).powi(2);
    }
    dist_sq.sqrt()
}

fn setup_distance(
    a: &GhostSetupCandidate,
    b: &GhostSetupCandidate,
    entity_positions: &HashMap<Entity, Position>,
) -> f32 {
    let breach_dist = score_distance(&a.spawn_point, &b.spawn_point);

    let extract_pos = |c: &GhostSetupCandidate, target_type| {
        c.influence_assignments
            .iter()
            .filter(|(_, t)| *t == target_type)
            .filter_map(|(e, _)| entity_positions.get(e))
            .copied()
            .collect::<Vec<_>>()
    };

    let a_attr = extract_pos(a, InfluenceType::Attractive);
    let b_attr = extract_pos(b, InfluenceType::Attractive);
    let dist_attr = signature_distance(
        &calculate_set_signature(&a_attr, &a.spawn_point),
        &calculate_set_signature(&b_attr, &b.spawn_point),
    );

    let a_rep = extract_pos(a, InfluenceType::Repulsive);
    let b_rep = extract_pos(b, InfluenceType::Repulsive);
    let dist_rep = signature_distance(
        &calculate_set_signature(&a_rep, &a.spawn_point),
        &calculate_set_signature(&b_rep, &b.spawn_point),
    );

    breach_dist + dist_attr + dist_rep
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
        distances.push(score_distance(ghost_spawn, obj_pos));
    }
    if let Some(pos) = player_pos {
        distances.push(score_distance(ghost_spawn, pos));
    }
    for (i, (_, _, pos1)) in influence_objects.iter().enumerate() {
        for (_, _, pos2) in influence_objects.iter().skip(i + 1) {
            distances.push(score_distance(pos1, pos2));
        }
    }

    if distances.is_empty() {
        return 0.0;
    }

    let sum_logs: f32 = distances.iter().map(|&d| (d + 1.0).ln()).sum();
    let n = distances.len() as f32;
    (sum_logs / n).exp()
}

pub(crate) fn generate_scored_ghost_setup(
    ghost_spawn_points: &[Position],
    objects_by_floor_with_positions: &HashMap<i64, Vec<(Entity, Position)>>,
    player_spawn_points: &[Position],
    board_toplogy: &BoardTopology,
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

    let mut candidates: Vec<GhostSetupCandidate> = Vec::with_capacity(EA_POPULATION_SIZE);
    let mut rng = random_seed::rng();

    for _ in 0..EA_POPULATION_SIZE {
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
        candidates.push(GhostSetupCandidate {
            spawn_point,
            influence_assignments,
            score,
        });
    }

    for _ in 0..EA_GENERATIONS {
        candidates.sort_by(|a, b| b.score.total_cmp(&a.score));

        let map_diagonal = ((board_toplogy.map_size.0.pow(2)
            + board_toplogy.map_size.1.pow(2)
            + board_toplogy.map_size.2.pow(2)) as f32)
            .sqrt()
            .max(10.0);

        // Sequential Niching (Fitness Sharing) to build diverse elites
        let mut i = 0;
        while i < EA_ELITE_COUNT && i < candidates.len() {
            let reference_elite = candidates[i].clone();

            for candidate in candidates.iter_mut().skip(i + 1) {
                let d = setup_distance(&reference_elite, candidate, &entity_positions);

                // Scale-invariant distance 0..1
                let normalized_dist = d / map_diagonal;

                // Smooth Cauchy-like similarity that drops accurately
                let similarity = 1.0 / (1.0 + (normalized_dist * 10.0).powi(2));

                let novelty_factor = (1.0 - similarity).clamp(0.0, 1.0);
                candidate.score *= novelty_factor;
            }

            // Re-sort ONLY the remainder so the next most highly scored AND diverse setup bubbles up
            candidates[(i + 1)..].sort_by(|a, b| b.score.total_cmp(&a.score));
            i += 1;
        }

        let mut next_generation = Vec::with_capacity(EA_POPULATION_SIZE);
        for candidate in candidates.iter().take(EA_ELITE_COUNT) {
            next_generation.push(candidate.clone());
        }

        for _ in EA_ELITE_COUNT..EA_POPULATION_SIZE {
            let parent_idx = rng.random_range(0..EA_ELITE_COUNT);
            let parent = &candidates[parent_idx];

            let mut child_spawn_point = parent.spawn_point;
            let mut child_influences = parent.influence_assignments.clone();

            if rng.random::<f32>() < EA_MUTATION_BREACH_CHANCE {
                child_spawn_point = select_ghost_spawn_point(ghost_spawn_points, &mut rng)
                    .unwrap_or(Position::new_i64(0, 0, 0));
            } else {
                child_influences =
                    select_influence_objects(&objects_by_floor, board_toplogy, &mut rng);
            }

            let mut influence_with_pos = Vec::new();
            for (entity, influence_type) in &child_influences {
                if let Some(position) = entity_positions.get(entity) {
                    influence_with_pos.push((*entity, *influence_type, *position));
                }
            }
            let score =
                score_ghost_setup(&child_spawn_point, &influence_with_pos, player_spawn_points);

            next_generation.push(GhostSetupCandidate {
                spawn_point: child_spawn_point,
                influence_assignments: child_influences,
                score,
            });
        }
        candidates = next_generation;
    }

    candidates.sort_by(|a, b| b.score.total_cmp(&a.score));
    let best_score = candidates[0].score;

    let chosen_setup = if DEBUG_SELECT_MAX {
        candidates.remove(0)
    } else {
        let elite_candidates = &candidates[0..EA_ELITE_COUNT];
        let total_score: f32 = elite_candidates.iter().map(|c| c.score.max(0.001)).sum();
        if total_score > 0.0 {
            let mut choice_value = rng.random::<f32>() * total_score;
            let mut chosen_idx = 0;
            for (i, candidate) in elite_candidates.iter().enumerate() {
                choice_value -= candidate.score.max(0.001);
                if choice_value <= 0.0 {
                    chosen_idx = i;
                    break;
                }
            }
            candidates.remove(chosen_idx)
        } else {
            candidates.remove(rng.random_range(0..EA_ELITE_COUNT))
        }
    };

    let elapsed = start_time.elapsed();
    debug!(
        "Ghost setup EVOLUTION completed: {} generations of {} pop in {:.2?}. Selected setup score: {:.2}. Best score: {:.2}. Debug max select: {}",
        EA_GENERATIONS,
        EA_POPULATION_SIZE,
        elapsed,
        chosen_setup.score,
        best_score,
        DEBUG_SELECT_MAX
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
    )
}
