use bevy::{prelude::*, window::PrimaryWindow};
use uncore::{
    components::{
        board::{PERSPECTIVE_X, PERSPECTIVE_Y, PERSPECTIVE_Z, position::Position},
        game::GCameraArena,
        move_to::MoveToTarget,
        player_sprite::PlayerSprite,
    },
    resources::{
        board_data::BoardData, mouse_visibility::MouseVisibility, player_input::PlayerInput,
    },
};

/// Converts screen coordinates to world coordinates using the game's isometric projection.
///
/// This function performs the inverse transformation of `Position::to_screen_coord()`,
/// solving a system of linear equations to determine world coordinates from screen position.
///
/// # Arguments
/// * `screen_pos` - The screen position (e.g., cursor position)
/// * `target_z` - The Z-level in world coordinates to project onto
/// * `camera` - The camera component for viewport-to-world conversion
/// * `camera_transform` - The camera's global transform
///
/// # Returns
/// * `Some(Position)` - The world position on the specified Z-level
/// * `None` - If the conversion fails (e.g., degenerate projection matrix)
fn screen_to_world_coords(
    screen_pos: Vec2,
    target_z: f32,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Position> {
    // Get the world position on the camera's near plane using Bevy's built-in conversion
    let world_pos_on_near_plane = camera
        .viewport_to_world_2d(camera_transform, screen_pos)
        .ok()?;

    // Calculate the determinant of the 2x2 isometric projection matrix
    let det = PERSPECTIVE_X[0] * PERSPECTIVE_Y[1] - PERSPECTIVE_Y[0] * PERSPECTIVE_X[1];
    if det.abs() < 1e-6 {
        return None; // Matrix is not invertible
    }
    let inv_det = 1.0 / det;

    // Adjust screen coordinates by removing the Z-level contribution
    let b_x = world_pos_on_near_plane.x - target_z * PERSPECTIVE_Z[0];
    let b_y = world_pos_on_near_plane.y - target_z * PERSPECTIVE_Z[1];

    // Apply the inverse transformation matrix to find world X and Y coordinates
    let world_x = inv_det * (b_x * PERSPECTIVE_Y[1] - PERSPECTIVE_Y[0] * b_y);
    let world_y = inv_det * (PERSPECTIVE_X[0] * b_y - b_x * PERSPECTIVE_X[1]);

    Some(Position {
        x: world_x,
        y: world_y,
        z: target_z,
        global_z: 0.0,
    })
}

/// System that handles click-to-move functionality using pathfinding.
///
/// When the player left-clicks on a walkable tile, this system converts the screen coordinates
/// to world coordinates and adds a `MoveToTarget` component to the player entity.
pub fn click_to_move_pathing_system(
    mut commands: Commands,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<GCameraArena>>,
    q_player: Query<(Entity, &Position), With<PlayerSprite>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_visibility: Res<MouseVisibility>,
    board_data: Res<BoardData>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // Only process clicks when mouse is visible
    if !mouse_visibility.is_visible {
        return;
    }

    let Ok(window) = q_window.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };
    let Ok((player_entity, player_pos)) = q_player.single() else {
        return;
    };

    // Convert cursor position to world coordinates
    let Some(target) = screen_to_world_coords(cursor_pos, player_pos.z, camera, camera_transform)
    else {
        return;
    };

    // Check if the target position is within bounds and walkable
    let board_pos = target.to_board_position();
    if !board_pos.is_valid(board_data.map_size) {
        return;
    }

    if let Some(collision) = board_data.collision_field.get(board_pos.ndidx()) {
        if collision.player_free {
            commands.entity(player_entity).insert(MoveToTarget(target));
        }
    }
}

/// System that updates player movement based on click-to-move target position.
///
/// This system reads the `MoveToTarget` component and converts it into movement input
/// by updating the `PlayerInput` resource. When the player reaches the target,
/// the `MoveToTarget` component is removed.
pub fn click_to_move_update_system(
    mut commands: Commands,
    q_player: Query<(Entity, &Position, &MoveToTarget)>,
    mut player_input: ResMut<PlayerInput>,
) {
    for (entity, pos, target) in q_player.iter() {
        let current = Vec2::new(pos.x, pos.y);
        let target_pos = Vec2::new(target.0.x, target.0.y);
        let to_target = target_pos - current;

        const ARRIVAL_THRESHOLD: f32 = 0.1;
        if to_target.length_squared() > ARRIVAL_THRESHOLD * ARRIVAL_THRESHOLD {
            // Move towards target
            player_input.movement = to_target.normalize();
        } else {
            // We've reached the target
            player_input.movement = Vec2::ZERO;
            commands.entity(entity).remove::<MoveToTarget>();
        }
    }
}
