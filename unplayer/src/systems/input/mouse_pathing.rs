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

/// Simple obstacle avoidance by checking if the straight path is clear.
///
/// If the target is valid but the path is blocked, this function finds alternative
/// positions that provide a clearer path to the destination.
fn find_walkable_target(
    player_pos: &Position,
    target_pos: Position,
    board_data: &BoardData,
) -> Option<Position> {
    let target_board_pos = target_pos.to_board_position();

    // First check if target is valid and walkable
    if !target_board_pos.is_valid(board_data.map_size) {
        return None;
    }

    let collision = board_data.collision_field.get(target_board_pos.ndidx())?;

    if !collision.player_free {
        return None;
    }

    // If target is walkable, check if path is clear
    if is_path_clear(player_pos, &target_pos, board_data) {
        return Some(target_pos);
    }

    // Path is blocked - find alternative positions around target that have clearer paths
    let search_radius = 2;
    let mut best_pos = None;
    let mut best_score = f32::MAX;

    for dx in -search_radius..=search_radius {
        for dy in -search_radius..=search_radius {
            if dx == 0 && dy == 0 {
                continue; // Skip the original target (already checked)
            }

            let test_board_pos = uncore::components::board::boardposition::BoardPosition {
                x: target_board_pos.x + dx,
                y: target_board_pos.y + dy,
                z: target_board_pos.z,
            };

            // Check if position is valid and walkable
            if !test_board_pos.is_valid(board_data.map_size) {
                continue;
            }

            if let Some(collision) = board_data.collision_field.get(test_board_pos.ndidx()) {
                if !collision.player_free {
                    continue;
                }
            } else {
                continue;
            }

            let test_pos = test_board_pos.to_position();

            // Check if path to this alternative position is clearer
            let path_clear = is_path_clear(player_pos, &test_pos, board_data);
            if !path_clear {
                continue;
            }

            // Score based on distance to original target (prefer positions close to original intent)
            let distance_to_target = target_pos.distance(&test_pos);
            if distance_to_target < best_score {
                best_score = distance_to_target;
                best_pos = Some(test_pos);
            }
        }
    }

    best_pos
}

/// Check if there's a clear path between two positions using simple line sampling.
///
/// This function samples points along the line between start and end positions
/// and checks if they're walkable. Returns false if any sampled point is blocked.
fn is_path_clear(start_pos: &Position, end_pos: &Position, board_data: &BoardData) -> bool {
    let start_2d = Vec2::new(start_pos.x, start_pos.y);
    let end_2d = Vec2::new(end_pos.x, end_pos.y);
    let direction = end_2d - start_2d;
    let distance = direction.length();

    // If very close, consider path clear
    if distance < 1.0 {
        return true;
    }

    // Sample along the path every 0.5 units
    let sample_step = 0.5;
    let num_samples = (distance / sample_step).ceil() as i32;

    for i in 1..num_samples {
        let t = i as f32 / num_samples as f32;
        let sample_point = start_2d + direction * t;

        let sample_pos = Position {
            x: sample_point.x,
            y: sample_point.y,
            z: start_pos.z, // Use same Z level
            global_z: 0.0,
        };

        let board_pos = sample_pos.to_board_position();

        // Check if this sample point is blocked
        if !board_pos.is_valid(board_data.map_size) {
            return false;
        }

        if let Some(collision) = board_data.collision_field.get(board_pos.ndidx()) {
            if !collision.player_free {
                return false;
            }
        }
    }

    true
}

/// System that handles click-to-move functionality with simple obstacle avoidance.
///
/// When the player left-clicks, this system converts the screen coordinates to world coordinates
/// and uses simple obstacle avoidance to find a walkable target position. If the clicked position
/// is blocked, it automatically finds the nearest walkable alternative within a small radius.
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

    // Use obstacle avoidance to find a walkable target position
    if let Some(walkable_target) = find_walkable_target(player_pos, target, &board_data) {
        commands
            .entity(player_entity)
            .insert(MoveToTarget::new(walkable_target));
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
        let target_pos = Vec2::new(target.position.x, target.position.y);
        let to_target = target_pos - current;

        const ARRIVAL_THRESHOLD: f32 = 0.1;
        if to_target.length_squared() > ARRIVAL_THRESHOLD * ARRIVAL_THRESHOLD {
            // Move towards target
            player_input.movement = to_target.normalize();
        } else {
            // We've reached the target
            player_input.movement = Vec2::ZERO;

            // Only remove MoveToTarget if there's no interaction to perform
            // If there's an interaction, let the complete_pending_interaction_system handle it
            if target.interaction_target.is_none() {
                commands.entity(entity).remove::<MoveToTarget>();
            }
        }
    }
}
