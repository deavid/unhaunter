use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::{
    components::{move_to::MoveToTarget, player_sprite::PlayerSprite},
    resources::player_input::PlayerInput,
};
use unsettings::game::{GameplaySettings, MovementStyle};

/// System that handles keyboard input for player movement.
///
/// This system reads keyboard input and converts it to movement vectors in the PlayerInput resource.
/// It also handles movement style transformations (e.g., screen-space orthogonal movement) and
/// clears any active click-to-move targets when keyboard movement is detected.
pub fn keyboard_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut player_input: ResMut<PlayerInput>,
    players: Query<(Entity, &PlayerSprite)>,
    game_settings: Res<Persistent<GameplaySettings>>,
) {
    for (entity, player) in players.iter() {
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

        // Normalize the movement vector if it's not zero
        if movement != Vec2::ZERO {
            movement = movement.normalize();
        }

        // Clear any click-to-move target when using keyboard
        if movement != Vec2::ZERO {
            commands.entity(entity).remove::<MoveToTarget>();
        }

        player_input.movement = movement;
    }
}
