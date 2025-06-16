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
            return reconstruct_path(&came_from, start_board, goal_board);
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
            return reconstruct_path(&came_from, start_board, goal_board);
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
}
