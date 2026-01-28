use bevy::prelude::*;
use bevy_persistent::Persistent;
use unnavigation_core::components::{
    move_to::MoveToTarget,
    waypoint::{Waypoint, WaypointOwner, WaypointQueue},
};
use unplayer_core::components::{MainPlayer, PlayerInput, PlayerSprite};
use unsettings_core::game::{GameplaySettings, MovementStyle};

/// System that handles keyboard input for player movement.
///
/// This system reads keyboard input and converts it to movement vectors in the PlayerInput component.
/// It also handles movement style transformations (e.g., screen-space orthogonal movement) and
/// clears any active click-to-move targets and waypoint queues when keyboard movement is detected.
pub(crate) fn keyboard_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut players: Query<(Entity, &PlayerSprite, &mut PlayerInput), With<MainPlayer>>,
    mut waypoint_queues: Query<&mut WaypointQueue>,
    q_existing_waypoints: Query<Entity, (With<Waypoint>, With<WaypointOwner>)>,
    game_settings: Res<Persistent<GameplaySettings>>,
) {
    for (entity, player, mut player_input) in players.iter_mut() {
        let mut movement = Vec2::ZERO;

        if keyboard_input.pressed(player.controls.up) {
            movement.y += 1.0;
        }
        if keyboard_input.pressed(player.controls.down) {
            movement.y -= 1.0;
        }
        if keyboard_input.pressed(player.controls.left) {
            movement.x -= 1.0;
        }
        if keyboard_input.pressed(player.controls.right) {
            movement.x += 1.0;
        }

        player_input.run = keyboard_input.pressed(player.controls.run);
        player_input.interact = keyboard_input.just_pressed(player.controls.activate);

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
        }

        player_input.movement = movement;
    }
}
