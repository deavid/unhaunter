use crate::components::board::boardposition::MapEntityFieldBPos;
use crate::components::board::position::Position;
use crate::components::game_config::GameConfig;
use crate::components::player_sprite::PlayerSprite;
use crate::resources::board_data::BoardData;
use bevy::prelude::*;

/// Synchronizes the map entity field with the current positions of entities.
///
/// This system updates the `BoardData` resource to reflect the current positions
/// of entities that have moved. It ensures that entities are correctly added to
/// and removed from the map entity field based on their new positions.
///
/// Optimized to only process entities within a reasonable radius of the player
/// using the map_entity_field for efficient entity lookup.
fn sync_map_entity_field(
    mut board_data: ResMut<BoardData>,
    game_config: Res<GameConfig>,
    player_query: Query<(&PlayerSprite, &Position)>,
    position_query: Query<&Position>,
    mut map_entity_bpos_query: Query<&mut MapEntityFieldBPos>,
) {
    let Some((_, player_pos)) = player_query
        .iter()
        .find(|(player, _)| player.id == game_config.player_id)
    else {
        return;
    };

    let player_bpos = player_pos.to_board_position();
    let (map_width, map_height, _map_depth) = board_data.map_size;

    // Define the update radius around player
    let update_radius: usize = 8;
    let min_x = player_bpos.ndidx().0.saturating_sub(update_radius);
    let max_x = (player_bpos.ndidx().0 + update_radius).min(map_width - 1);
    let min_y = player_bpos.ndidx().1.saturating_sub(update_radius);
    let max_y = (player_bpos.ndidx().1 + update_radius).min(map_height - 1);
    let z = player_bpos.ndidx().2;

    let mut to_update = vec![];

    // Process entities within the update radius
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            let entities = &board_data.map_entity_field[(x, y, z)];

            for &entity in entities.iter() {
                // Check if entity has a Position component
                if let Ok(current_pos) = position_query.get(entity) {
                    let current_bpos = current_pos.to_board_position();

                    // Only update if the entity has a MapEntityFieldBPos component
                    if let Ok(mut old_bpos) = map_entity_bpos_query.get_mut(entity) {
                        if old_bpos.0 != current_bpos {
                            to_update.push((entity, current_bpos.clone(), old_bpos.0.clone()));

                            // info!(
                            //     "Moved entity {:?} from {:?} to {:?}",
                            //     entity,
                            //     (x, y, z),
                            //     current_bpos.ndidx()
                            // );
                            // Update the stored BoardPosition
                            old_bpos.0 = current_bpos;
                        }
                    }
                }
            }
        }
    }
    for (entity, current_bpos, old_bpos) in to_update {
        // Remove from the current position in the map_entity_field
        if let Some(entity_vec) = board_data.map_entity_field.get_mut(old_bpos.ndidx()) {
            entity_vec.retain(|&e| e != entity);
        }

        // Add to the new position
        if let Some(entity_vec) = board_data.map_entity_field.get_mut(current_bpos.ndidx()) {
            entity_vec.push(entity);
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, sync_map_entity_field);
}
