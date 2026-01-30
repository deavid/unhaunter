use crate::direction::Direction;
use crate::position::Position;
use bevy::prelude::*;

// old perspective (9x20cm) const SUBTL: f32 = 9.0; new perspective (3x20cm)
pub const SUBTL: f32 = 3.0;

// new perspective (3x20cm) - reduced const SUBTL: f32 = 2.5;
pub const PERSPECTIVE_X: [f32; 3] = [4.0 * SUBTL, -2.0 * SUBTL, 0.0001];
pub const PERSPECTIVE_Y: [f32; 3] = [4.0 * SUBTL, 2.0 * SUBTL, -0.0001];
pub const PERSPECTIVE_Z: [f32; 3] = [0.0, 4.0 * 11.0, 0.01];

pub fn to_screen_coord(pos: Position) -> Vec3 {
    let x = pos.x * PERSPECTIVE_X[0] + pos.y * PERSPECTIVE_Y[0] + pos.z * PERSPECTIVE_Z[0];
    let y = pos.x * PERSPECTIVE_X[1] + pos.y * PERSPECTIVE_Y[1] + pos.z * PERSPECTIVE_Z[1];
    let z = pos.x * PERSPECTIVE_X[2] + pos.y * PERSPECTIVE_Y[2] + pos.z.round() * PERSPECTIVE_Z[2];
    Vec3::new(x, y, z + pos.visual_priority)
}

pub fn direction_to_screen_coord(dir: Direction) -> Vec3 {
    let x = dir.dx * PERSPECTIVE_X[0] + dir.dy * PERSPECTIVE_Y[0] + dir.dz * PERSPECTIVE_Z[0];
    let y = dir.dx * PERSPECTIVE_X[1] + dir.dy * PERSPECTIVE_Y[1] + dir.dz * PERSPECTIVE_Z[1];
    let z = dir.dx * PERSPECTIVE_X[2] + dir.dy * PERSPECTIVE_Y[2] + dir.dz * PERSPECTIVE_Z[2];
    Vec3::new(x, y, z)
}

/// Converts a 2D screen position (like the cursor) to a 3D world position
/// on a specific Z-plane.
pub fn screen_to_world(
    screen_pos: Vec2,
    target_z: f32,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Position> {
    // Get the world position on the camera's near plane
    let Ok(world_pos_on_near_plane) = camera.viewport_to_world_2d(camera_transform, screen_pos)
    else {
        return None;
    };

    // This is the reverse of the projection math in `Position::to_screen_coord`.
    // We are solving a system of two linear equations for world_x and world_y,
    // given screen_x, screen_y, and a fixed world_z.
    //
    // screen.x = world.x * P_X.x + world.y * P_Y.x + world.z * P_Z.x
    // screen.y = world.x * P_X.y + world.y * P_Y.y + world.z * P_Z.y

    // The 2x2 matrix for our projection is:
    // [ PERSPECTIVE_X.x  PERSPECTIVE_Y.x ]
    // [ PERSPECTIVE_X.y  PERSPECTIVE_Y.y ]
    let det = PERSPECTIVE_X[0] * PERSPECTIVE_Y[1] - PERSPECTIVE_Y[0] * PERSPECTIVE_X[1];

    if det.abs() < 1e-6 {
        return None; // The matrix is not invertible, which shouldn't happen with your projection.
    }
    let inv_det = 1.0 / det;

    // First, adjust the screen coordinates by the amount contributed by the fixed Z-level.
    let b_x = world_pos_on_near_plane.x - target_z * PERSPECTIVE_Z[0];
    let b_y = world_pos_on_near_plane.y - target_z * PERSPECTIVE_Z[1];

    // Now, apply the inverse matrix to find world_x and world_y.
    let world_x = inv_det * (b_x * PERSPECTIVE_Y[1] - PERSPECTIVE_Y[0] * b_y);
    let world_y = inv_det * (PERSPECTIVE_X[0] * b_y - b_x * PERSPECTIVE_X[1]);

    Some(Position {
        x: world_x,
        y: world_y,
        z: target_z,
        visual_priority: 0.0, // Aiming is on the logical plane.
    })
}
