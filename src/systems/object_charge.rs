//! This module defines systems related to managing the charge levels of objects that influence ghost behavior.

use crate::board::Position;
use crate::components::ghost_influence::{GhostInfluence, InfluenceType};
use crate::ghost::GhostSprite;
use crate::object_interaction::ObjectInteractionConfig;
use bevy::prelude::*;

/// System to accumulate charge on objects over time.
fn accumulate_charge(
    config: Res<ObjectInteractionConfig>, // Access the object interaction configuration
    time: Res<Time>,                      // Access the time resource
    mut query: Query<&mut GhostInfluence>, // Query for mutable GhostInfluence components
) {
    let delta_time = time.delta_seconds(); // Get the time elapsed since the last frame

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
fn check_ghost_proximity(
    config: Res<ObjectInteractionConfig>, // Access the object interaction configuration
    ghost_query: Query<&Position, With<GhostSprite>>, // Query for the ghost's position
    object_query: Query<(Entity, &Position, &GhostInfluence)>, // Query for object positions and GhostInfluence
    mut commands: Commands, // Access commands to add/remove components
) {
    // Get ghost position
    let Ok(ghost_position) = ghost_query.get_single() else {
        return;
    };

    // Iterate through objects with GhostInfluence
    for (entity, object_position, _) in &object_query {
        // Calculate distance between object and ghost
        let distance = ghost_position.distance(object_position);

        // If distance <= object_discharge_radius
        if distance <= config.object_discharge_radius {
            // Add WithinDischargeRange component to the object entity
            commands.entity(entity).insert(WithinDischargeRange);
        } else {
            commands.entity(entity).remove::<WithinDischargeRange>();
        }
    }
}

/// System to discharge objects within the ghost's range based on their influence type.
fn discharge_objects(
    config: Res<ObjectInteractionConfig>,
    time: Res<Time>,
    mut query: Query<&mut GhostInfluence, With<WithinDischargeRange>>, // Query for mutable GhostInfluence components of objects within discharge range
) {
    let delta_time = time.delta_seconds();

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
