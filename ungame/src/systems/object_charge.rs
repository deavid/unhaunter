//! This module defines systems related to managing the charge levels of objects
//! that influence ghost behavior.
use uncore::components::board::position::Position;
use uncore::components::ghost_influence::{GhostInfluence, InfluenceType};
use uncore::resources::object_interaction::ObjectInteractionConfig;

use crate::difficulty::CurrentDifficulty;
use crate::ghost::GhostSprite;

use bevy::prelude::*;
use bevy::utils::HashSet;

/// System to accumulate charge on objects over time.
fn accumulate_charge(
    // Access the object interaction configuration
    config: Res<ObjectInteractionConfig>,
    // Access the time resource
    time: Res<Time>,
    // Query for mutable GhostInfluence components
    mut query: Query<&mut GhostInfluence>,
) {
    // Get the time elapsed since the last frame
    let delta_time = time.delta_secs();

    // Iterate through entities with GhostInfluence
    for mut ghost_influence in &mut query {
        // Increase charge_value by object_charge_rate * delta_time
        ghost_influence.charge_value += config.object_charge_rate * delta_time;

        // Clamp charge_value to a maximum of 1.0
        ghost_influence.charge_value = ghost_influence.charge_value.clamp(0.0, 1.0);
    }
}

/// Component to mark objects within the ghost's discharge range
#[derive(Component)]
struct WithinDischargeRange;

/// System to check ghost proximity to objects
#[allow(clippy::too_many_arguments)]
fn check_ghost_proximity(
    // Access the object interaction configuration
    config: Res<ObjectInteractionConfig>,
    // Query for the ghost's position
    mut ghost_query: Query<(&Position, &mut GhostSprite)>,
    // Query for object positions and GhostInfluence
    object_query: Query<(Entity, &Position, &GhostInfluence)>,
    // FIXME: This parameter would not reset between missions.
    mut removed_attractive_objects: Local<HashSet<Entity>>,
    // Access commands to add/remove components
    mut commands: Commands,
    // Access the room database
    roomdb: Res<crate::uncore_board::RoomDB>,
    // Access the time resource
    time: Res<Time>,
    // Access the difficulty settings
    difficulty: Res<CurrentDifficulty>,
) {
    // Get ghost position and breach position
    let Ok((ghost_position, mut ghost_sprite)) = ghost_query.get_single_mut() else {
        return;
    };
    let breach_position = ghost_sprite.spawn_point.to_position();

    // Iterate through objects with GhostInfluence
    for (entity, object_position, ghost_influence) in &object_query {
        // Calculate distance between object and ghost
        let distance_to_ghost = ghost_position.distance(object_position);

        // If distance <= object_discharge_radius
        if distance_to_ghost <= config.object_discharge_radius {
            // Add WithinDischargeRange component to the object entity
            commands.entity(entity).insert(WithinDischargeRange);

            // --- Hunt Provocation Logic ---
            if ghost_influence.influence_type == InfluenceType::Repulsive {
                // Calculate distance between object and breach
                let distance_to_breach = breach_position.distance(object_position);

                // If distance <= hunt_provocation_radius and charge_value is above threshold:
                if distance_to_breach <= difficulty.0.hunt_provocation_radius
                    && ghost_influence.charge_value > 0.8
                {
                    ghost_sprite.rage += 0.2;
                }
            }
        } else {
            commands.entity(entity).remove::<WithinDischargeRange>();

            // --- Check for Removed Attractive Objects ---
            if ghost_influence.influence_type == InfluenceType::Attractive {
                // If the object was previously within range but is now outside, mark it as removed
                if removed_attractive_objects.contains(&entity) {
                    continue;
                }

                // Remove the object from the list of removed objects if it's back within range
                if roomdb
                    .room_tiles
                    .get(&object_position.to_board_position())
                    .is_some()
                {
                    removed_attractive_objects.remove(&entity);
                } else {
                    // Add the object to the list of removed objects
                    removed_attractive_objects.insert(entity);
                }
            }
        }
    }

    // --- Increase Anger for Removed Attractive Objects ---
    let delta_time = time.delta_secs();
    for _ in removed_attractive_objects.iter() {
        ghost_sprite.rage += difficulty.0.attractive_removal_anger_rate * delta_time;
    }
}

/// System to discharge objects within the ghost's range based on their influence
/// type.
fn discharge_objects(
    config: Res<ObjectInteractionConfig>,
    time: Res<Time>,
    // Query for mutable GhostInfluence components of objects within discharge range
    mut query: Query<&mut GhostInfluence, With<WithinDischargeRange>>,
) {
    let delta_time = time.delta_secs();
    for mut ghost_influence in &mut query {
        match ghost_influence.influence_type {
            InfluenceType::Attractive => {
                ghost_influence.charge_value -= config.attractive_discharge_multiplier * delta_time;
            }
            InfluenceType::Repulsive => {
                ghost_influence.charge_value -= config.repulsive_discharge_multiplier * delta_time;
            }
        }

        // Clamp charge_value to a minimum of 0.0
        ghost_influence.charge_value = ghost_influence.charge_value.clamp(0.0, 1.0);
    }
}

/// Adds the object charge management systems to the Bevy app.
pub fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (accumulate_charge, check_ghost_proximity, discharge_objects).chain(),
    );
}
