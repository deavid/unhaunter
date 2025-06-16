use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use bevy::prelude::*;
use uncore::{
    components::board::{boardposition::BoardPosition, position::Position},
    resources::board_data::BoardData,
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

/// Gets valid neighboring positions for pathfinding
fn get_neighbors(pos: &BoardPosition, board_data: &BoardData) -> Vec<BoardPosition> {
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

        // Check if the neighbor is within bounds and walkable
        if let Some(idx) = neighbor.ndidx_checked(board_data.map_size) {
            if let Some(collision_data) = board_data.collision_field.get(idx) {
                if collision_data.player_free {
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
pub fn find_path(start: Position, goal: Position, board_data: &BoardData) -> Vec<BoardPosition> {
    let start_board = start.to_board_position();
    let goal_board = goal.to_board_position();

    // Early exit if start equals goal
    if start_board == goal_board {
        return vec![start_board];
    }

    // Check if start and goal positions are valid and walkable
    if let Some(start_idx) = start_board.ndidx_checked(board_data.map_size) {
        if let Some(start_collision) = board_data.collision_field.get(start_idx) {
            if !start_collision.player_free {
                warn!("Start position {:?} is not walkable", start_board);
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
            return smooth_path(raw_path, board_data);
        }

        // Move current to closed set
        closed_set.insert(current_pos.clone());

        // Check all neighbors
        for neighbor in get_neighbors(&current_pos, board_data) {
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
) -> Vec<BoardPosition> {
    let start_board = start.to_board_position();
    let goal_board = goal.to_board_position();

    // Early exit if start equals goal
    if start_board == goal_board {
        return vec![start_board];
    }

    // Check if start position is valid and walkable
    if let Some(start_idx) = start_board.ndidx_checked(board_data.map_size) {
        if let Some(start_collision) = board_data.collision_field.get(start_idx) {
            if !start_collision.player_free {
                warn!("Start position {:?} is not walkable", start_board);
                return Vec::new();
            }
        }
    } else {
        warn!("Start position {:?} is out of bounds", start_board);
        return Vec::new();
    }

    // Check if goal position is within bounds (but don't check walkability - treat as walkable)
    if goal_board.ndidx_checked(board_data.map_size).is_none() {
        warn!("Goal position {:?} is out of bounds", goal_board);
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
            return smooth_path(raw_path, board_data);
        }

        // Move current to closed set
        closed_set.insert(current_pos.clone());

        // Check all neighbors - use special function that treats goal as walkable
        for neighbor in get_neighbors_to_interactive(&current_pos, board_data, &goal_board) {
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
            // If this is the goal position, always treat it as walkable
            if neighbor == *goal {
                neighbors.push(neighbor);
            } else if let Some(collision_data) = board_data.collision_field.get(idx) {
                // For non-goal positions, check walkability normally
                if collision_data.player_free {
                    neighbors.push(neighbor);
                }
            }
        }
    }

    neighbors
}

/// Smooths a path by removing unnecessary waypoints using line-of-sight checks.
/// This creates more natural-looking paths that move diagonally when possible
/// while still avoiding collisions.
pub fn smooth_path(path: Vec<BoardPosition>, board_data: &BoardData) -> Vec<BoardPosition> {
    if path.len() <= 2 {
        return path; // Can't smooth paths with 2 or fewer points
    }

    let mut smoothed = vec![path[0].clone()]; // Always keep start position
    let mut current_index = 0;

    while current_index < path.len() - 1 {
        // Find the furthest point we can see from current position
        let mut furthest_visible = current_index + 1;

        for i in (current_index + 2)..path.len() {
            if has_line_of_sight(&path[current_index], &path[i], board_data) {
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
fn has_line_of_sight(from: &BoardPosition, to: &BoardPosition, board_data: &BoardData) -> bool {
    // If positions are the same, there's always line of sight
    if from == to {
        return true;
    }

    // If positions are adjacent (Manhattan distance 1), check direct connection
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();
    if dx <= 1 && dy <= 1 && dx + dy <= 2 {
        // Adjacent or diagonal neighbors - check if destination is walkable
        return is_walkable(to, board_data);
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

        if !is_walkable(&sample_pos, board_data) {
            return false;
        }
    }

    // Also check the final destination
    is_walkable(to, board_data)
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

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array3;
    use uncore::types::board::fielddata::CollisionFieldData;

    fn create_test_board_data(size: (usize, usize, usize)) -> BoardData {
        let mut board_data = BoardData::from_world(&mut bevy::prelude::World::new());
        board_data.map_size = size;
        board_data.collision_field = Array3::from_elem(
            size,
            CollisionFieldData {
                player_free: true,
                ..Default::default()
            },
        );
        board_data
    }

    #[test]
    fn test_pathfinding_simple() {
        let board_data = create_test_board_data((10, 10, 1));

        let start = Position::new_i64(0, 0, 0);
        let goal = Position::new_i64(3, 3, 0);

        let path = find_path(start, goal, &board_data);

        assert!(!path.is_empty());
        assert_eq!(path[0], BoardPosition { x: 0, y: 0, z: 0 });
        assert_eq!(path[path.len() - 1], BoardPosition { x: 3, y: 3, z: 0 });
    }

    #[test]
    fn test_pathfinding_no_path() {
        let mut board_data = create_test_board_data((5, 5, 1));

        // Create a wall blocking the path
        for y in 0..5 {
            let idx = (2, y, 0);
            board_data.collision_field[idx].player_free = false;
        }

        let start = Position::new_i64(0, 0, 0);
        let goal = Position::new_i64(4, 0, 0);

        let path = find_path(start, goal, &board_data);

        assert!(path.is_empty());
    }

    #[test]
    fn test_pathfinding_interactive() {
        let board_data = create_test_board_data((10, 10, 1));

        let start = Position::new_i64(0, 0, 0);
        let goal = Position::new_i64(3, 3, 0);

        let path = find_path_to_interactive(start, goal, &board_data);

        assert!(!path.is_empty());
        assert_eq!(path[0], BoardPosition { x: 0, y: 0, z: 0 });
        assert_eq!(path[path.len() - 1], BoardPosition { x: 3, y: 3, z: 0 });
    }

    #[test]
    fn test_pathfinding_interactive_goal_blocked() {
        let mut board_data = create_test_board_data((5, 5, 1));

        // Create a wall at the goal position
        let goal_position = Position::new_i64(4, 4, 0);
        if let Some(idx) = goal_position
            .to_board_position()
            .ndidx_checked(board_data.map_size)
        {
            board_data.collision_field[idx].player_free = false;
        }

        let start = Position::new_i64(0, 0, 0);
        let goal = goal_position;

        let path = find_path_to_interactive(start, goal, &board_data);

        assert!(!path.is_empty());
        assert_eq!(path[0], BoardPosition { x: 0, y: 0, z: 0 });
        assert_eq!(path[path.len() - 1], goal_position.to_board_position());
    }

    #[test]
    fn test_path_smoothing() {
        let board_data = create_test_board_data((10, 10, 1));

        // Create a zig-zag path
        let mut path = Vec::new();
        for i in 0..5 {
            path.push(BoardPosition { x: i, y: 0, z: 0 });
            path.push(BoardPosition { x: i, y: 1, z: 0 });
        }

        // Smooth the path
        let smoothed_path = smooth_path(path.clone(), &board_data);

        // Check that the smoothed path is shorter and still valid
        assert!(smoothed_path.len() < path.len());
        assert_eq!(smoothed_path[0], path[0]);
        assert_eq!(smoothed_path[smoothed_path.len() - 1], path[path.len() - 1]);

        // Check that all positions in the smoothed path are walkable
        for pos in smoothed_path.iter() {
            assert!(is_walkable(pos, &board_data));
        }
    }

    #[test]
    fn test_line_of_sight() {
        let board_data = create_test_board_data((10, 10, 1));

        // Direct line, should have line of sight
        let from = BoardPosition { x: 0, y: 0, z: 0 };
        let to = BoardPosition { x: 5, y: 0, z: 0 };
        assert!(has_line_of_sight(&from, &to, &board_data));

        // Blocked by wall, should not have line of sight
        let mut blocked_board_data = board_data.clone();
        let wall_pos = BoardPosition { x: 3, y: 0, z: 0 };
        if let Some(idx) = wall_pos.ndidx_checked(blocked_board_data.map_size) {
            blocked_board_data.collision_field[idx].player_free = false;
        }
        assert!(!has_line_of_sight(&from, &to, &blocked_board_data));

        // Diagonal line, should have line of sight if no walls
        let to_diag = BoardPosition { x: 3, y: 3, z: 0 };
        assert!(has_line_of_sight(&from, &to_diag, &board_data));

        // Diagonal line blocked by wall, should not have line of sight
        let diag_wall_pos = BoardPosition { x: 2, y: 2, z: 0 };
        if let Some(idx) = diag_wall_pos.ndidx_checked(blocked_board_data.map_size) {
            blocked_board_data.collision_field[idx].player_free = false;
        }
        assert!(!has_line_of_sight(&from, &to_diag, &blocked_board_data));
    }
}
