use bevy::{prelude::*, window::PrimaryWindow};
use unclassic_mode_core::components::GCameraArena;
use uninput_core::components::PlayerInput;
use uninput_core::resources::MouseVisibility;
use unplayer_core::components::{MainPlayer, PlayerSprite};
use unspatial_core::direction::Direction;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;

const AIM_MAX_DISTANCE: f32 = 12.0;

pub fn mouse_aim_system(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<GCameraArena>>,
    mut q_player: Query<
        (&mut Direction, &Position, &mut PlayerInput),
        (With<PlayerSprite>, With<MainPlayer>),
    >,
    mouse_visibility: Res<MouseVisibility>,
    q_in_truck: Query<(), (With<MainPlayer>, With<InTruck>)>,
) {
    // Skip if local player is in truck
    if !q_in_truck.is_empty() {
        return;
    }
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
    let Ok((mut player_dir, player_pos, mut player_input)) = q_player.single_mut() else {
        return;
    };

    // Use our new conversion function, fixing the Z-plane to the player's Z.
    if let Some(mouse_world_pos) =
        perspective::screen_to_world(cursor_pos, player_pos.z, camera, cam_transform)
    {
        // The vector from the player to the mouse in world coordinates.
        let aim_vec = mouse_world_pos.delta(*player_pos);
        let clamped_aim_vec = aim_vec.with_max_dist(AIM_MAX_DISTANCE) * 30.0;

        // This is now the correct aiming direction.
        // NOTE: Intentional dual-write. Direction is the authoritative spatial state that consumers
        // (locomotion, AI) depend on. aim_direction on PlayerInput acts as the replication-safe bus
        // copy for networking. Both writes must be kept in sync.
        *player_dir = clamped_aim_vec;
        player_input.aim_direction = Vec2::new(clamped_aim_vec.dx, clamped_aim_vec.dy);
    }
}
