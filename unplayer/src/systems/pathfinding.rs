use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use bevy::prelude::*;
use uncore::{
    behavior::component::Stairs,
    behavior::{Behavior, Orientation},
    components::board::{boardposition::BoardPosition, position::Position},
    resources::{board_data::BoardData, visibility_data::VisibilityData},
};

/// A* pathfinding node for the priority queue
#[derive(Debug, Clone, PartialEq, Eq)]
struct PathNode {
    position: BoardPosition,
    f_cost: i32, // g_cost + h_cost
    g_cost: i32, // Distance from start
    h_cost: i32, // Heuristic distance to goal
}

impl PathNode {
    fn new(position: BoardPosition, g_cost: i32, h_cost: i32) -> Self {
        Self {
            position,
            f_cost: g_cost + h_cost,
            g_cost,
            h_cost,
        }
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other
            .f_cost
            .cmp(&self.f_cost)
            .then_with(|| other.h_cost.cmp(&self.h_cost))
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Calculates Manhattan distance heuristic between two board positions
fn heuristic(a: &BoardPosition, b: &BoardPosition) -> i32 {
    ((a.x - b.x).abs() + (a.y - b.y).abs()) as i32
}

/// Helper function to check if a board position is visible to the player
fn is_visible(
    pos: &BoardPosition,
    board_data: &BoardData,
    visibility_data: &VisibilityData,
) -> bool {
    if let Some(idx) = pos.ndidx_checked(board_data.map_size) {
        if let Some(visibility) = visibility_data.visibility_field.get(idx) {
            return *visibility > 0.0; // Assume visibility > 0 means visible
        }
    }
    false // Out of bounds or no visibility data means not visible
}

/// Gets valid neighboring positions for pathfinding
fn get_neighbors(
    pos: &BoardPosition,
    board_data: &BoardData,
    visibility_data: &VisibilityData,
) -> Vec<BoardPosition> {
    let mut neighbors = Vec::new();

    // Check 4-directional movement (up, down, left, right)
    let directions = [
        (0, -1), // Up
        (0, 1),  // Down
        (-1, 0), // Left
        (1, 0),  // Right
    ];

    for (dx, dy) in directions {
        let neighbor = BoardPosition {
            x: pos.x + dx,
            y: pos.y + dy,
            z: pos.z, // Stay on same floor for now
        };

        // Check if the neighbor is within bounds, walkable, and visible
        if let Some(idx) = neighbor.ndidx_checked(board_data.map_size) {
            if let Some(collision_data) = board_data.collision_field.get(idx) {
                if collision_data.player_free && is_visible(&neighbor, board_data, visibility_data)
                {
                    neighbors.push(neighbor);
                }
            }
        }
    }

    neighbors
}

/// Reconstructs the path from the came_from map
fn reconstruct_path(
    came_from: &HashMap<BoardPosition, BoardPosition>,
    start: BoardPosition,
    goal: BoardPosition,
) -> Vec<BoardPosition> {
    let mut path = Vec::new();
    let mut current = goal.clone();

    // Build path backwards from goal to start
    while current != start {
        path.push(current.clone());
        if let Some(parent) = came_from.get(&current) {
            current = parent.clone();
        } else {
            // Path reconstruction failed
            warn!("Path reconstruction failed at position {:?}", current);
            return Vec::new();
        }
    }

    // Add start position and reverse to get forward path
    path.push(start);
    path.reverse();
    path
}

/// Performs A* pathfinding from start to goal position
/// Returns a vector of BoardPositions representing the path, or empty vector if no path found
pub fn find_path(
    start: Position,
    goal: Position,
    board_data: &BoardData,
    visibility_data: &VisibilityData,
) -> Vec<BoardPosition> {
    let start_board = start.to_board_position();
    let goal_board = goal.to_board_position();

    // Early exit if start equals goal
    if start_board == goal_board {
        return vec![start_board];
    }

    // Check if start and goal positions are valid, walkable, and visible
    if let Some(start_idx) = start_board.ndidx_checked(board_data.map_size) {
        if let Some(start_collision) = board_data.collision_field.get(start_idx) {
            if !start_collision.player_free {
                warn!("Start position {:?} is not walkable", start_board);
                return Vec::new();
            }
            if !is_visible(&start_board, board_data, visibility_data) {
                warn!("Start position {:?} is not visible", start_board);
                return Vec::new();
            }
        }
    } else {
        warn!("Start position {:?} is out of bounds", start_board);
        return Vec::new();
    }

    if let Some(goal_idx) = goal_board.ndidx_checked(board_data.map_size) {
        if let Some(goal_collision) = board_data.collision_field.get(goal_idx) {
            if !goal_collision.player_free {
                warn!("Goal position {:?} is not walkable", goal_board);
                return Vec::new();
            }
            if !is_visible(&goal_board, board_data, visibility_data) {
                warn!("Goal position {:?} is not visible", goal_board);
                return Vec::new();
            }
        }
    } else {
        warn!("Goal position {:?} is out of bounds", goal_board);
        return Vec::new();
    }

    // A* algorithm implementation
    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashSet::new();
    let mut came_from = HashMap::new();
    let mut g_costs = HashMap::new();

    // Initialize with start position
    let start_h = heuristic(&start_board, &goal_board);
    open_set.push(PathNode::new(start_board.clone(), 0, start_h));
    g_costs.insert(start_board.clone(), 0);

    while let Some(current_node) = open_set.pop() {
        let current_pos = current_node.position.clone();

        // Check if we reached the goal
        if current_pos == goal_board {
            let raw_path = reconstruct_path(&came_from, start_board, goal_board);
            return smooth_path(raw_path, board_data, visibility_data);
        }

        // Move current to closed set
        closed_set.insert(current_pos.clone());

        // Check all neighbors
        for neighbor in get_neighbors(&current_pos, board_data, visibility_data) {
            if closed_set.contains(&neighbor) {
                continue;
            }

            let tentative_g = current_node.g_cost + 1; // Movement cost is always 1

            let neighbor_g = g_costs.get(&neighbor).copied().unwrap_or(i32::MAX);

            if tentative_g < neighbor_g {
                // Found a better path to this neighbor
                came_from.insert(neighbor.clone(), current_pos.clone());
                g_costs.insert(neighbor.clone(), tentative_g);

                let h_cost = heuristic(&neighbor, &goal_board);
                open_set.push(PathNode::new(neighbor, tentative_g, h_cost));
            }
        }
    }

    // No path found
    info!("No path found from {:?} to {:?}", start_board, goal_board);
    Vec::new()
}

/// Performs A* pathfinding from start to goal position for interactive objects.
/// Unlike find_path, this function treats the goal position as walkable even if it has collision,
/// which is useful for pathfinding to interactive objects like closed doors.
/// Returns a vector of BoardPositions representing the path, or empty vector if no path found
pub fn find_path_to_interactive(
    start: Position,
    goal: Position,
    board_data: &BoardData,
    visibility_data: &VisibilityData,
) -> Vec<BoardPosition> {
    let start_board = start.to_board_position();
    let goal_board = goal.to_board_position();

    // Early exit if start equals goal
    if start_board == goal_board {
        return vec![start_board];
    }

    // Check if start position is valid, walkable, and visible
    if let Some(start_idx) = start_board.ndidx_checked(board_data.map_size) {
        if let Some(start_collision) = board_data.collision_field.get(start_idx) {
            if !start_collision.player_free {
                warn!("Start position {:?} is not walkable", start_board);
                return Vec::new();
            }
            if !is_visible(&start_board, board_data, visibility_data) {
                warn!("Start position {:?} is not visible", start_board);
                return Vec::new();
            }
        }
    } else {
        warn!("Start position {:?} is out of bounds", start_board);
        return Vec::new();
    }

    // Check if goal position is within bounds and visible (but don't check walkability - treat as walkable)
    if goal_board.ndidx_checked(board_data.map_size).is_none() {
        warn!("Goal position {:?} is out of bounds", goal_board);
        return Vec::new();
    }
    if !is_visible(&goal_board, board_data, visibility_data) {
        warn!("Goal position {:?} is not visible", goal_board);
        return Vec::new();
    }

    // A* algorithm implementation with special handling for goal position
    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashSet::new();
    let mut came_from = HashMap::new();
    let mut g_costs = HashMap::new();

    // Initialize with start position
    let start_h = heuristic(&start_board, &goal_board);
    open_set.push(PathNode::new(start_board.clone(), 0, start_h));
    g_costs.insert(start_board.clone(), 0);

    while let Some(current_node) = open_set.pop() {
        let current_pos = current_node.position.clone();

        // Check if we reached the goal
        if current_pos == goal_board {
            let raw_path = reconstruct_path(&came_from, start_board, goal_board);
            return smooth_path(raw_path, board_data, visibility_data);
        }

        // Move current to closed set
        closed_set.insert(current_pos.clone());

        // Check all neighbors - use special function that treats goal as walkable
        for neighbor in
            get_neighbors_to_interactive(&current_pos, board_data, visibility_data, &goal_board)
        {
            if closed_set.contains(&neighbor) {
                continue;
            }

            let tentative_g = current_node.g_cost + 1; // Movement cost is always 1

            let neighbor_g = g_costs.get(&neighbor).copied().unwrap_or(i32::MAX);

            if tentative_g < neighbor_g {
                // Found a better path to this neighbor
                came_from.insert(neighbor.clone(), current_pos.clone());
                g_costs.insert(neighbor.clone(), tentative_g);

                let h_cost = heuristic(&neighbor, &goal_board);
                open_set.push(PathNode::new(neighbor, tentative_g, h_cost));
            }
        }
    }

    // No path found
    info!("No path found from {:?} to {:?}", start_board, goal_board);
    Vec::new()
}

/// Gets valid neighboring positions for pathfinding to interactive objects.
/// Treats the goal position as walkable even if it has collision.
fn get_neighbors_to_interactive(
    pos: &BoardPosition,
    board_data: &BoardData,
    visibility_data: &VisibilityData,
    goal: &BoardPosition,
) -> Vec<BoardPosition> {
    let mut neighbors = Vec::new();

    // Check 4-directional movement (up, down, left, right)
    let directions = [
        (0, -1), // Up
        (0, 1),  // Down
        (-1, 0), // Left
        (1, 0),  // Right
    ];

    for (dx, dy) in directions {
        let neighbor = BoardPosition {
            x: pos.x + dx,
            y: pos.y + dy,
            z: pos.z, // Stay on same floor for now
        };

        // Check if the neighbor is within bounds
        if let Some(idx) = neighbor.ndidx_checked(board_data.map_size) {
            // If this is the goal position, always treat it as walkable (but still check visibility)
            if neighbor == *goal {
                if is_visible(&neighbor, board_data, visibility_data) {
                    neighbors.push(neighbor);
                }
            } else if let Some(collision_data) = board_data.collision_field.get(idx) {
                // For non-goal positions, check walkability and visibility normally
                if collision_data.player_free && is_visible(&neighbor, board_data, visibility_data)
                {
                    neighbors.push(neighbor);
                }
            }
        }
    }

    neighbors
}

/// Smooths a path by removing unnecessary waypoints using line-of-sight checks.
/// This creates more natural-looking paths that move diagonally when possible
/// while still avoiding collisions and invisible areas.
pub fn smooth_path(
    path: Vec<BoardPosition>,
    board_data: &BoardData,
    visibility_data: &VisibilityData,
) -> Vec<BoardPosition> {
    if path.len() <= 2 {
        return path; // Can't smooth paths with 2 or fewer points
    }

    let mut smoothed = vec![path[0].clone()]; // Always keep start position
    let mut current_index = 0;

    while current_index < path.len() - 1 {
        // Find the furthest point we can see from current position
        let mut furthest_visible = current_index + 1;

        for i in (current_index + 2)..path.len() {
            if has_line_of_sight(&path[current_index], &path[i], board_data, visibility_data) {
                furthest_visible = i;
            } else {
                break; // Can't see further, stop here
            }
        }

        smoothed.push(path[furthest_visible].clone());
        current_index = furthest_visible;
    }

    smoothed
}

/// Checks if there's a clear line of sight between two board positions.
/// Uses a simple ray-casting approach to sample points along the line.
/// Also checks visibility in addition to collision.
fn has_line_of_sight(
    from: &BoardPosition,
    to: &BoardPosition,
    board_data: &BoardData,
    visibility_data: &VisibilityData,
) -> bool {
    // If positions are the same, there's always line of sight
    if from == to {
        return true;
    }

    // If positions are adjacent (Manhattan distance 1), check direct connection
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();
    if dx <= 1 && dy <= 1 && dx + dy <= 2 {
        // Adjacent or diagonal neighbors - check if destination is walkable and visible
        return is_walkable_and_visible(to, board_data, visibility_data);
    }

    // For longer distances, sample points along the line
    let distance = ((to.x - from.x).pow(2) + (to.y - from.y).pow(2)) as f64;
    let distance = distance.sqrt();

    // Sample every 0.5 units along the line
    let sample_count = (distance * 2.0).ceil() as i32;

    for i in 1..sample_count {
        let t = i as f64 / sample_count as f64;
        let sample_x = from.x as f64 + (to.x - from.x) as f64 * t;
        let sample_y = from.y as f64 + (to.y - from.y) as f64 * t;

        let sample_pos = BoardPosition {
            x: sample_x.round() as i64,
            y: sample_y.round() as i64,
            z: from.z, // Stay on same Z level
        };

        if !is_walkable_and_visible(&sample_pos, board_data, visibility_data) {
            return false;
        }
    }

    // Also check the final destination
    is_walkable_and_visible(to, board_data, visibility_data)
}

/// Helper function to check if a board position is walkable
fn is_walkable(pos: &BoardPosition, board_data: &BoardData) -> bool {
    if let Some(idx) = pos.ndidx_checked(board_data.map_size) {
        if let Some(collision_data) = board_data.collision_field.get(idx) {
            return collision_data.player_free;
        }
    }
    false
}

/// Helper function to check if a board position is both walkable and visible
fn is_walkable_and_visible(
    pos: &BoardPosition,
    board_data: &BoardData,
    visibility_data: &VisibilityData,
) -> bool {
    is_walkable(pos, board_data) && is_visible(pos, board_data, visibility_data)
}

/// Detects if a position is within a stairs area and returns stair information
pub fn detect_stair_area(
    target_pos: Position,
    stairs_query: &Query<(Entity, &Position, &Stairs, &Behavior)>,
) -> Option<(Entity, Position, Stairs, Behavior, Position, Position)> {
    let target_bpos = target_pos.to_board_position();

    info!(
        "Checking if position {:?} (board: {:?}) is in stairs area",
        target_pos, target_bpos
    );

    for (stair_entity, stair_pos, stair_component, behavior) in stairs_query.iter() {
        let stair_bpos = stair_pos.to_board_position();

        info!(
            "Checking stair at {:?} (board: {:?}), orientation: {:?}, z: {}",
            stair_pos,
            stair_bpos,
            behavior.orientation(),
            stair_component.z
        );

        // Check if target is within the stairs area based on orientation
        let in_stairs_area = match behavior.orientation() {
            Orientation::XAxis => {
                // XAxis stairs: 2 tiles in X, 3 tiles in Y
                (stair_bpos.x == target_bpos.x || stair_bpos.x + 1 == target_bpos.x)
                    && (target_bpos.y - stair_bpos.y).abs() <= 1
                    && stair_bpos.z == target_bpos.z
            }
            Orientation::YAxis => {
                // YAxis stairs: 2 tiles in Y, 3 tiles in X
                (stair_bpos.y == target_bpos.y || stair_bpos.y - 1 == target_bpos.y)
                    && (target_bpos.x - stair_bpos.x).abs() <= 1
                    && stair_bpos.z == target_bpos.z
            }
            _ => false,
        };

        if in_stairs_area {
            info!(
                "Found stair area! Entity: {:?}, Position: {:?}, Z: {}, Orientation: {:?}",
                stair_entity,
                stair_pos,
                stair_component.z,
                behavior.orientation()
            );
            // Calculate start and end waypoints for stair traversal
            let (start_waypoint, end_waypoint) =
                calculate_stair_waypoints(stair_pos, stair_component, behavior, stairs_query);
            info!(
                "Calculated waypoints: start={:?}, end={:?}",
                start_waypoint, end_waypoint
            );
            return Some((
                stair_entity,
                *stair_pos,
                stair_component.clone(),
                behavior.clone(),
                start_waypoint,
                end_waypoint,
            ));
        }
    }

    info!("No stair area found for position {:?}", target_pos);
    None
}

/// Detects the actual direction of stairs by analyzing the paired stair
/// Returns true if the stairs go in the "positive" direction (for XAxis: +Y, for YAxis: +X)
/// Returns false if the stairs go in the "negative" direction (for XAxis: -Y, for YAxis: -X)
fn detect_stair_direction(
    stair_pos: &Position,
    stair_component: &Stairs,
    behavior: &Behavior,
    stairs_query: &Query<(Entity, &Position, &Stairs, &Behavior)>,
) -> bool {
    let target_z = stair_pos.z + stair_component.z as f32;
    let stair_bpos = stair_pos.to_board_position();

    // Look for paired stairs on the target floor
    for (_paired_entity, paired_pos, paired_component, paired_behavior) in stairs_query.iter() {
        let paired_bpos = paired_pos.to_board_position();

        // Check if this could be a paired stair
        if paired_bpos.z as f32 != target_z {
            continue;
        }

        // Paired stairs should have opposite Z direction
        if paired_component.z != -stair_component.z {
            continue;
        }

        // Paired stairs should have same orientation
        if paired_behavior.orientation() != behavior.orientation() {
            continue;
        }

        match behavior.orientation() {
            Orientation::XAxis => {
                // For XAxis stairs, check if X coordinates are close and Y direction
                if (paired_bpos.x - stair_bpos.x).abs() <= 2 {
                    // If paired stair is at higher Y, stairs go in positive Y direction
                    return paired_bpos.y > stair_bpos.y;
                }
            }
            Orientation::YAxis => {
                // For YAxis stairs, check if Y coordinates are close and X direction
                if (paired_bpos.y - stair_bpos.y).abs() <= 2 {
                    // If paired stair is at higher X, stairs go in positive X direction
                    return paired_bpos.x > stair_bpos.x;
                }
            }
            _ => {}
        }
    }

    // Default to positive direction if no paired stair found
    true
}

/// Calculates the start and end waypoints for traversing stairs
/// Based on the stairs_player system logic in keyboard.rs
pub fn calculate_stair_waypoints(
    stair_pos: &Position,
    stair_component: &Stairs,
    behavior: &Behavior,
    stairs_query: &Query<(Entity, &Position, &Stairs, &Behavior)>,
) -> (Position, Position) {
    // Detect the actual direction of the stairs
    let positive_direction =
        detect_stair_direction(stair_pos, stair_component, behavior, stairs_query);
    match behavior.orientation() {
        Orientation::XAxis => {
            // For XAxis stairs, movement is in Y direction
            // Start: at the beginning of the stair area (no Z change)
            // End: at the end where Z transition is complete

            let (start_y, end_y) = if positive_direction {
                // Stairs go in positive Y direction (normal)
                if stair_component.z > 0 {
                    // Going up: start at bottom, end at top + offset
                    (stair_pos.y - 2.0, stair_pos.y + 2.0)
                } else {
                    // Going down: start at top, end at bottom + offset
                    // This one is possibly unused.
                    panic!("this case should not happen")
                    // dbg!(stair_pos.y + 1.0, stair_pos.y - 1.0)
                }
            } else {
                // Stairs go in negative Y direction (mirrored)
                if stair_component.z > 0 {
                    // Going up: start at top, end at bottom + offset
                    // This one is possibly unused.
                    panic!("this case should not happen")
                    // dbg!(stair_pos.y + 1.0, stair_pos.y - 1.0)
                } else {
                    // Going down: start at bottom, end at top + offset
                    (stair_pos.y + 1.0, stair_pos.y - 3.0)
                }
            };

            let start_waypoint = Position {
                x: stair_pos.x,
                y: start_y,
                z: stair_pos.z, // Same floor
                global_z: 0.0,
            };

            let end_waypoint = Position {
                x: stair_pos.x,
                y: end_y,
                z: stair_pos.z + stair_component.z as f32, // New floor
                global_z: 0.0,
            };

            (start_waypoint, end_waypoint)
        }
        Orientation::YAxis => {
            // For YAxis stairs, movement is in X direction
            let (start_x, end_x) = if positive_direction {
                // Stairs go in positive X direction (normal)
                if stair_component.z > 0 {
                    // Going up: start at left, end at right + offset
                    // dbg!(stair_pos.x - 1.0, stair_pos.x + 1.0)
                    panic!("this case should not happen")
                } else {
                    // Going down: start at right, end at left + offset
                    dbg!(stair_pos.x - 1.0, stair_pos.x + 3.0)
                }
            } else {
                // Stairs go in negative X direction (mirrored)
                if stair_component.z > 0 {
                    // Going up: start at right, end at left + offset
                    dbg!(stair_pos.x + 2.0, stair_pos.x - 2.0)
                } else {
                    // Going down: start at left, end at right + offset
                    // dbg!(stair_pos.x - 1.0, stair_pos.x + 1.0)
                    panic!("this case should not happen")
                }
            };

            let start_waypoint = Position {
                x: start_x,
                y: stair_pos.y,
                z: stair_pos.z, // Same floor
                global_z: 0.0,
            };

            let end_waypoint = Position {
                x: end_x,
                y: stair_pos.y,
                z: stair_pos.z + stair_component.z as f32, // New floor
                global_z: 0.0,
            };

            (start_waypoint, end_waypoint)
        }
        _ => {
            // Fallback for unsupported orientations
            (*stair_pos, *stair_pos)
        }
    }
}
