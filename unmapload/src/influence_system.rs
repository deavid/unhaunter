//! # Ghost Influence System Module
//!
//! This module handles the assignment of ghost influence properties to objects in the game.
//! Ghost influence affects how objects interact with paranormal activity, making them
//! either attractive or repulsive to ghost energy.

use bevy::prelude::*;
use rand::seq::SliceRandom;
use uncore::components::board::position::Position;
use uncore::components::ghost_influence::{GhostInfluence, InfluenceType};
use uncore::random_seed;
use uncore::resources::roomdb::RoomDB;

use crate::level_setup::AssignGhostInfluenceMarker;

/// Assigns GhostInfluence components to movable objects that are inside rooms.
///
/// This system runs after level loading is complete to ensure room data is available.
/// It selects up to 3 objects from valid rooms and assigns different influence types to them:
/// - One repulsive object (rejects ghost energy)
/// - Up to two attractive objects (attract ghost energy)
///
/// This creates interesting interactions where ghosts affect certain objects more than others.
///
/// # Arguments
/// * `commands` - Command buffer for entity modifications
/// * `marker_query` - Query to find the marker component with movable object list
/// * `position_query` - Query to get positions of objects
/// * `roomdb` - Room database to identify objects within valid rooms
pub fn assign_ghost_influence_system(
    mut commands: Commands,
    marker_query: Query<(Entity, &AssignGhostInfluenceMarker)>,
    position_query: Query<&Position>,
    roomdb: Res<RoomDB>,
) {
    for (marker_entity, marker) in marker_query.iter() {
        let mut valid_movable_objects = Vec::new();

        // Filter objects to only include those in rooms
        for &entity in &marker.0 {
            if let Ok(pos) = position_query.get(entity) {
                let board_pos = pos.to_board_position();

                // Check if the object is in a valid room
                if roomdb.room_tiles.contains_key(&board_pos) {
                    valid_movable_objects.push(entity);
                }
            }
        }

        // Log warning if no valid objects found
        if valid_movable_objects.is_empty() {
            warn!(
                "No movable objects found in valid rooms. Ghost influence system will not work properly."
            );
            commands.entity(marker_entity).despawn();
            continue;
        }

        // Shuffle the valid movable objects for randomization
        let mut rng = random_seed::rng();
        valid_movable_objects.shuffle(&mut rng);

        // Select up to 3 objects from the valid ones
        let selected_objects = valid_movable_objects.iter().take(3);

        // Assign GhostInfluence components with different types
        for (i, &entity) in selected_objects.enumerate() {
            let influence_type = if i == 0 {
                // First object is repulsive (rejects ghost energy)
                InfluenceType::Repulsive
            } else {
                // Others are attractive (attract ghost energy)
                InfluenceType::Attractive
            };

            // Add the GhostInfluence component to the entity
            commands.entity(entity).insert(GhostInfluence {
                influence_type,
                charge_value: 0.0,
            });
        }

        info!("Successfully assigned ghost influence properties to objects in valid rooms");

        // Remove the marker entity as its job is done
        commands.entity(marker_entity).despawn();
    }
}
