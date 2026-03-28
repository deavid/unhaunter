use bevy::prelude::*;
use bevy_persistent::Persistent;
use uninput_core::components::{PlayerInput, PlayerInputMapping};
use uninput_core::resources::MissionInputFocus;
use unnavigation_core::components::{
    move_to::MoveToTarget,
    waypoint::{Waypoint, WaypointOwner, WaypointQueue},
};
use unplayer_core::components::{MainPlayer, PlayerSprite};
use unsettings_core::game::{GameplaySettings, MovementStyle};

/// System that handles keyboard input for player movement.
///
/// This system reads keyboard input and converts it to movement vectors in the PlayerInput component.
/// It also handles movement style transformations (e.g., screen-space orthogonal movement) and
/// clears any active click-to-move targets and waypoint queues when keyboard movement is detected.
pub fn keyboard_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut players: Query<
        (Entity, &PlayerSprite, &PlayerInputMapping, &mut PlayerInput),
        With<MainPlayer>,
    >,
    mut waypoint_queues: Query<&mut WaypointQueue>,
    q_existing_waypoints: Query<Entity, (With<Waypoint>, With<WaypointOwner>)>,
    game_settings: Res<Persistent<GameplaySettings>>,
    focus: Res<MissionInputFocus>,
) {
    if !focus.has_focus {
        return;
    }
    for (entity, _player, input_mapping, mut player_input) in players.iter_mut() {
        let mut movement = Vec2::ZERO;

        if keyboard_input.pressed(input_mapping.controls.up) {
            movement.y += 1.0;
        }
        if keyboard_input.pressed(input_mapping.controls.down) {
            movement.y -= 1.0;
        }
        if keyboard_input.pressed(input_mapping.controls.left) {
            movement.x -= 1.0;
        }
        if keyboard_input.pressed(input_mapping.controls.right) {
            movement.x += 1.0;
        }

        player_input.run = keyboard_input.pressed(input_mapping.controls.run);
        player_input.interact = keyboard_input.just_pressed(input_mapping.controls.activate);
        player_input.hide_requested = keyboard_input.pressed(input_mapping.controls.activate);
        player_input.unhide_requested =
            keyboard_input.just_pressed(input_mapping.controls.activate);
        player_input.grab = keyboard_input.just_pressed(input_mapping.controls.grab);
        player_input.drop = keyboard_input.just_pressed(input_mapping.controls.drop);
        player_input.use_right_hand = keyboard_input
            .just_pressed(input_mapping.controls.right_hand_trigger)
            || mouse_input.just_pressed(MouseButton::Right);
        player_input.use_left_hand =
            keyboard_input.just_pressed(input_mapping.controls.left_hand_trigger);
        player_input.inventory_cycle = keyboard_input.just_pressed(input_mapping.controls.cycle);
        player_input.inventory_swap = keyboard_input.just_pressed(input_mapping.controls.swap);

        // Apply MovementStyle transformation (from original keyboard_player)
        if matches!(
            game_settings.movement_style,
            MovementStyle::ScreenSpaceOrthogonal
        ) {
            const PERSPECTIVE_X: [f32; 2] = [1.0, 1.0];
            const PERSPECTIVE_Y: [f32; 2] = [-1.0, 1.0];
            let od = movement;
            movement.x = od.x * PERSPECTIVE_X[0] + od.y * PERSPECTIVE_Y[0];
            movement.y = od.x * PERSPECTIVE_X[1] + od.y * PERSPECTIVE_Y[1];
        }

        if movement != Vec2::ZERO {
            // Normalize the movement vector if it's not zero
            movement = movement.normalize();

            // Clear any click-to-move target and waypoint queue when using keyboard
            commands.entity(entity).remove::<MoveToTarget>();

            // Clear waypoint queue and despawn waypoint entities
            if let Ok(mut waypoint_queue) = waypoint_queues.get_mut(entity) {
                // Despawn all waypoint entities belonging to this player
                for waypoint_entity in &waypoint_queue.0 {
                    if q_existing_waypoints.contains(*waypoint_entity) {
                        commands.entity(*waypoint_entity).despawn();
                    }
                }
                waypoint_queue.clear();
            }

            // Keyboard is active, set movement direction
            player_input.movement = movement;
        } else {
            // No keyboard input. Only zero out movement if there are no active waypoints.
            // If waypoints are active, waypoint_following_system will set movement.
            // Writing zero here would race with client_send_input_system on the client,
            // which is in a separate chain and may read the zero before waypoint_following
            // can overwrite it.
            let has_waypoints = waypoint_queues
                .get(entity)
                .map(|q| !q.is_empty())
                .unwrap_or(false);
            if !has_waypoints {
                player_input.movement = movement;
            }
        }
    }
}
