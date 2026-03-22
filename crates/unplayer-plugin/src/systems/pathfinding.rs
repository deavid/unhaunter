use bevy::prelude::*;
use unbehavior_core::behavior::Behavior;
use unbehavior_core::components::Stairs;
use unspatial_core::orientation::Orientation;
use unspatial_core::position::Position;

/// Detects if a position is within a stairs area and returns stair information
pub(crate) fn detect_stair_area(
    target_pos: Position,
    stairs_query: &Query<(Entity, &Position, &Stairs, &Behavior)>,
) -> Option<(Entity, Position, Stairs, Behavior, Position, Position)> {
    let target_bpos = target_pos.to_board_position();

    for (stair_entity, stair_pos, stair_component, behavior) in stairs_query.iter() {
        let stair_bpos = stair_pos.to_board_position();

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
            trace!(
                "Found stair area! Entity: {:?}, Position: {:?}, Z: {}, Orientation: {:?}",
                stair_entity,
                stair_pos,
                stair_component.z,
                behavior.orientation()
            );
            // Calculate start and end waypoints for stair traversal
            let (start_waypoint, end_waypoint) =
                calculate_stair_waypoints(stair_pos, stair_component, behavior, stairs_query);
            trace!(
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
pub(crate) fn calculate_stair_waypoints(
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
                }
            } else {
                // Stairs go in negative Y direction (mirrored)
                if stair_component.z > 0 {
                    // Going up: start at top, end at bottom + offset
                    // This one is possibly unused.
                    panic!("this case should not happen")
                } else {
                    // Going down: start at bottom, end at top + offset
                    (stair_pos.y + 1.0, stair_pos.y - 3.0)
                }
            };

            let start_waypoint = Position {
                x: stair_pos.x,
                y: start_y,
                z: stair_pos.z, // Same floor
                visual_priority: 0.0,
            };

            let end_waypoint = Position {
                x: stair_pos.x,
                y: end_y,
                z: stair_pos.z + stair_component.z as f32, // New floor
                visual_priority: 0.0,
            };

            (start_waypoint, end_waypoint)
        }
        Orientation::YAxis => {
            // For YAxis stairs, movement is in X direction
            let (start_x, end_x) = if positive_direction {
                // Stairs go in positive X direction (normal)
                if stair_component.z > 0 {
                    // Going up: start at left, end at right + offset
                    panic!("this case should not happen")
                } else {
                    // Going down: start at right, end at left + offset
                    (stair_pos.x - 1.0, stair_pos.x + 3.0)
                }
            } else {
                // Stairs go in negative X direction (mirrored)
                if stair_component.z > 0 {
                    // Going up: start at right, end at left + offset
                    (stair_pos.x + 2.0, stair_pos.x - 2.0)
                } else {
                    // Going down: start at left, end at right + offset
                    panic!("this case should not happen")
                }
            };

            let start_waypoint = Position {
                x: start_x,
                y: stair_pos.y,
                z: stair_pos.z, // Same floor
                visual_priority: 0.0,
            };

            let end_waypoint = Position {
                x: end_x,
                y: stair_pos.y,
                z: stair_pos.z + stair_component.z as f32, // New floor
                visual_priority: 0.0,
            };

            (start_waypoint, end_waypoint)
        }
        _ => {
            // Fallback for unsupported orientations
            (*stair_pos, *stair_pos)
        }
    }
}
