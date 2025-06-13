use bevy::{prelude::*, window::PrimaryWindow};
use uncore::{
    components::{
        board::{
            PERSPECTIVE_X, PERSPECTIVE_Y, PERSPECTIVE_Z, direction::Direction, position::Position,
        },
        game::GCameraArena,
        player_sprite::PlayerSprite,
    },
    resources::mouse_visibility::MouseVisibility,
    states::GameState,
};

const AIM_MAX_DISTANCE: f32 = 8.0;

/// Converts a 2D screen position (like the cursor) to a 3D world position
/// on a specific Z-plane.
fn screen_to_world(
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
        global_z: 0.0, // Aiming is on the logical plane.
    })
}

fn mouse_aim_system(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<GCameraArena>>,
    mut q_player: Query<(&mut Direction, &Position), With<PlayerSprite>>,
    mouse_visibility: Res<MouseVisibility>,
) {
    // Only aim when mouse is visible
    if !mouse_visibility.is_visible {
        return;
    }
    let Ok(window) = q_window.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_transform)) = q_camera.single() else {
        return;
    };
    let Ok((mut player_dir, player_pos)) = q_player.single_mut() else {
        return;
    };

    // Use our new conversion function, fixing the Z-plane to the player's Z.
    if let Some(mouse_world_pos) = screen_to_world(cursor_pos, player_pos.z, camera, cam_transform)
    {
        // The vector from the player to the mouse in world coordinates.
        let aim_vec = mouse_world_pos.delta(*player_pos);
        let clamped_aim_vec = aim_vec.with_max_dist(AIM_MAX_DISTANCE) * 20.0;

        // This is now the correct aiming direction.
        *player_dir = clamped_aim_vec;
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, mouse_aim_system.run_if(in_state(GameState::None)));
}
