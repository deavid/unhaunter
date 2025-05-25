//! # Ghost Influence System Module
//!
//! This module handles the assignment of ghost influence properties to objects in the game.
//! Ghost influence affects how objects interact with paranormal activity, making them
//! either attractive or repulsive to ghost energy.

use bevy::prelude::*;
use bevy::utils::HashMap;
use uncore::components::board::position::Position;
use uncore::components::ghost_influence::GhostInfluence;
use uncore::resources::board_data::BoardData;
use uncore::resources::roomdb::RoomDB;

use crate::level_setup::AssignGhostInfluenceMarker;

/// Assigns GhostInfluence components to movable objects that are inside rooms.
///
/// This system runs after level loading is complete to ensure room data is available.
/// It handles both the general case and floor-specific requirements by using the
/// selection module to determine which objects should receive influence components.
///
/// # Arguments
/// * `commands` - Command buffer for entity modifications
/// * `marker_query` - Query to find the marker component with movable object list
/// * `position_query` - Query to get positions of objects
/// * `roomdb` - Room database to identify objects within valid rooms
/// * `board_data` - Board data for accessing floor properties
fn assign_ghost_influence_system(
    mut commands: Commands,
    marker_query: Query<(Entity, &AssignGhostInfluenceMarker)>,
    ghost_spawn_query: Query<&Position, With<uncore::components::ghost_breach::GhostBreach>>,
    player_spawn_query: Query<&Position, With<uncore::components::player_sprite::PlayerSprite>>,
    position_query: Query<&Position>,
    roomdb: Res<RoomDB>,
    board_data: Res<BoardData>,
) {
    for (marker_entity, marker) in marker_query.iter() {
        // Get all objects that are in valid rooms, organized by floor with positions
        let mut objects_by_floor_with_positions: HashMap<i64, Vec<(Entity, Position)>> =
            HashMap::new();

        // Get player spawn positions for spatial distribution calculations
        let player_positions: Vec<Position> = player_spawn_query.iter().copied().collect();

        // Get ghost spawn positions - if ghost is already spawned, use its position
        let mut ghost_spawn_points = Vec::new();
        if let Ok(ghost_pos) = ghost_spawn_query.get_single() {
            ghost_spawn_points.push(*ghost_pos);
        } else {
            // Fallback to breach position if no ghost is spawned yet
            ghost_spawn_points.push(board_data.breach_pos);
        }

        // Organize movable objects by floor, including positions
        for &entity in &marker.0 {
            if let Ok(pos) = position_query.get(entity) {
                let board_pos = pos.to_board_position();

                // Check if the object is in a valid room
                if roomdb.room_tiles.contains_key(&board_pos) {
                    // Get the floor Z coordinate
                    let floor_z = board_pos.z;

                    // Add this object to the appropriate floor's list with its position
                    objects_by_floor_with_positions
                        .entry(floor_z)
                        .or_default()
                        .push((entity, *pos));
                }
            }
        }

        // If no valid objects are found
        if objects_by_floor_with_positions.is_empty() {
            warn!(
                "No movable objects found in valid rooms. Ghost influence system will not work properly."
            );
            commands.entity(marker_entity).despawn();
            continue;
        }

        // Start timing the simulation
        let start = bevy::utils::Instant::now();

        // Use the simulation-based selection function to determine optimal setup
        let (_, selected_objects) = crate::selection::select_influence_objects_with_simulation(
            &objects_by_floor_with_positions,
            &ghost_spawn_points,
            &player_positions,
            &board_data,
        );

        let elapsed = start.elapsed();
        info!(
            "Simulation-based ghost influence selection completed in {:.2?}",
            elapsed
        );

        // Assign the components to all selected objects
        for (entity, influence_type) in selected_objects {
            commands.entity(entity).insert(GhostInfluence {
                influence_type,
                charge_value: 0.0,
            });
        }

        // Remove the marker entity as its job is done
        commands.entity(marker_entity).despawn();
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, assign_ghost_influence_system);
}
