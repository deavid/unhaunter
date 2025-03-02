use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use ndarray::Array3;
use std::collections::VecDeque;
use uncore::{
    behavior::{Behavior, Class, TileState},
    components::board::{boardposition::BoardPosition, position::Position},
    resources::board_data::BoardData,
    types::board::{
        fielddata::LightFieldData,
        prebaked_lighting_data::{PrebakedLightingData, PropagationDirection},
    },
};

/// Helper function to check if there are active light sources nearby
#[allow(dead_code)]
pub fn has_active_light_nearby(
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
pub fn is_light_active(pos: &BoardPosition, behaviors: &HashMap<BoardPosition, &Behavior>) -> bool {
    if let Some(behavior) = behaviors.get(pos) {
        behavior.p.light.emits_light
    } else {
        false
    }
}

/// Checks if a position is within the board boundaries
pub fn is_in_bounds(pos: (i64, i64, i64), map_size: (usize, usize, usize)) -> bool {
    pos.0 >= 0
        && pos.1 >= 0
        && pos.2 >= 0
        && pos.0 < map_size.0 as i64
        && pos.1 < map_size.1 as i64
        && pos.2 < map_size.2 as i64
}

/// Gets the relative neighbor position from a direction
#[allow(dead_code)]
pub fn get_direction_offset(dir: PropagationDirection) -> (i64, i64, i64) {
    match dir {
        PropagationDirection::North => (0, 1, 0),
        PropagationDirection::East => (1, 0, 0),
        PropagationDirection::South => (0, -1, 0),
        PropagationDirection::West => (-1, 0, 0),
    }
}

/// Checks if light can propagate in a specific direction from the given position
#[allow(dead_code)]
pub fn can_propagate_in_direction(
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
pub fn blend_colors(
    c1: (f32, f32, f32),
    lux1: f32,
    c2: (f32, f32, f32),
    lux2: f32,
) -> (f32, f32, f32) {
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
pub fn identify_active_light_sources(
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
pub fn apply_prebaked_contributions(
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
pub fn update_exposure_and_stats(bf: &mut BoardData, lfs: &Array3<LightFieldData>) {
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
pub fn identify_portal_points(bf: &BoardData) -> HashSet<(usize, usize, usize)> {
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
pub fn collect_door_states(
    qt: &Query<(&Position, &Behavior)>,
) -> HashMap<(usize, usize, usize), bool> {
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
pub fn prepare_continuation_points(
    bf: &BoardData,
    active_source_ids: &HashSet<u32>,
    door_states: &HashMap<(usize, usize, usize), bool>,
    portal_points: &HashSet<(usize, usize, usize)>,
) -> Vec<(BoardPosition, u32, f32, f32, (f32, f32, f32))> {
    let mut continuation_points = Vec::new();

    // STEP 1: Handle standard continuation points from prebaking
    for ((i, j, k), prebaked_data) in bf.prebaked_lighting.indexed_iter() {
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

        // KEY FIX: Process this position if:
        // 1. It's an open door OR
        // 2. It's a portal point OR
        // 3. It's a continuation point with pending propagations
        if (is_door_position && is_open_door) || is_portal || prebaked_data.is_continuation_point {
            // FIXME: code reaches here perfectly multiple times
            // For each pending propagation from active sources
            // warn!(
            //     "Continuation point found at: {:?}, {}, {:?}",
            //     pos,
            //     prebaked_data.pending_propagations.len(),
            //     active_source_ids
            // );
            // prebaked_data.pending_propagations.len is zero.
            for pending in &prebaked_data.pending_propagations {
                if active_source_ids.contains(&pending.source_id) {
                    // FIXME: But this is never executed
                    warn!("Propagation point found at: {:?}", pos);
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
    }

    info!("Prepared {} continuation points", continuation_points.len());
    continuation_points
}

/// Adds dynamic light sources to the lighting field
pub fn add_dynamic_light_sources(
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
pub fn propagate_from_continuation_points(
    bf: &BoardData,
    lfs: &mut Array3<LightFieldData>,
    visited: &mut Array3<bool>,
    continuation_points: &[(BoardPosition, u32, f32, f32, (f32, f32, f32))],
    active_source_ids: &HashSet<u32>,
    door_states: &HashMap<(usize, usize, usize), bool>,
    portal_points: &HashSet<(usize, usize, usize)>,
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
        if remaining_distance <= 0.0 || current_lux < 0.0000001 {
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
            // Check if this position is a door and it's open
            let current_pos_idx = (pos.x as usize, pos.y as usize, pos.z as usize);
            let current_is_open_door = door_states.get(&current_pos_idx).copied().unwrap_or(false);

            // Calculate neighbor position
            let nx = pos.x + dx;
            let ny = pos.y + dy;
            let nz = pos.z + dz;

            // Skip if out of bounds
            if !is_in_bounds((nx, ny, nz), bf.map_size) {
                continue;
            }

            let neighbor_pos = (nx as usize, ny as usize, nz as usize);

            // Get prebaked propagation directions
            let standard_can_propagate = match dir_i {
                0 => prebaked_data.propagation_dirs.north,
                1 => prebaked_data.propagation_dirs.east,
                2 => prebaked_data.propagation_dirs.south,
                3 => prebaked_data.propagation_dirs.west,
                _ => false,
            };

            let is_open_door = door_states.get(&neighbor_pos).copied().unwrap_or(false);
            let is_portal = portal_points.contains(&neighbor_pos);

            // Check if we can propagate based on:
            // 1. Normal prebaked propagation OR
            // 2. We're at an open door (allow propagation in all directions) OR
            // 3. Neighbor is an open door OR portal point
            let can_propagate =
                standard_can_propagate || current_is_open_door || is_open_door || is_portal;

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
