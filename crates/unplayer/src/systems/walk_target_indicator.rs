use crate::components::walk_target_indicator::WalkTargetIndicator;
use bevy::prelude::*;
use uncore::components::{
    board::position::Position, move_to::MoveToTarget, player_sprite::PlayerSprite,
};

/// System that manages the walk target indicator.
/// Spawns a red dot when a player has a MoveToTarget component,
/// and despawns it when the MoveToTarget component is removed.
pub fn manage_walk_target_indicator(
    mut commands: Commands,
    player_query: Query<&MoveToTarget, (With<PlayerSprite>, Without<WalkTargetIndicator>)>,
    mut indicator_query: Query<(Entity, &mut Position), With<WalkTargetIndicator>>,
    move_target_exists_query: Query<&MoveToTarget, With<PlayerSprite>>,
) {
    let has_move_target = move_target_exists_query.iter().next().is_some();
    let indicator_exists = indicator_query.iter().next().is_some();

    if has_move_target && !indicator_exists {
        // Player has a move target but no indicator exists - spawn one
        if let Ok(move_target) = player_query.single() {
            let target_position = move_target.position;

            commands
                .spawn(Sprite {
                    color: Color::srgba(1.0, 0.0, 0.0, 0.3),
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                })
                .insert(Position {
                    x: target_position.x,
                    y: target_position.y,
                    z: target_position.z,
                    global_z: target_position.global_z + 0.01,
                })
                .insert(WalkTargetIndicator);
        }
    } else if !has_move_target && indicator_exists {
        for (indicator_entity, _) in indicator_query.iter() {
            commands.entity(indicator_entity).despawn();
        }
    } else if has_move_target && indicator_exists {
        // Player has a move target and the indicator exists - update indicator's position
        if let Ok(move_target) = move_target_exists_query.single()
            && let Ok((_indicator_entity, mut indicator_position)) = indicator_query.single_mut()
        {
            indicator_position.x = move_target.position.x;
            indicator_position.y = move_target.position.y;
            indicator_position.z = move_target.position.z;
            indicator_position.global_z = move_target.position.global_z + 0.01;
        }
    }
}
