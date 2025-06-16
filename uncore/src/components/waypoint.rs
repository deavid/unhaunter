use bevy::prelude::*;

/// Component that marks an entity as a waypoint.
/// Waypoint entities serve as both data containers and visual representations.
#[derive(Component, Debug)]
pub struct Waypoint {
    pub waypoint_type: WaypointType,
    pub order: u32,
}

/// The type of action to perform when reaching this waypoint.
#[derive(Debug, Clone)]
pub enum WaypointType {
    /// Just move to this position
    MoveTo,
    /// Move to this position and then interact with the specified entity
    Interact(Entity),
}

/// Component on player entities that tracks their waypoint queue.
/// Contains an ordered list of waypoint entities to follow.
#[derive(Component, Debug, Default)]
pub struct WaypointQueue(pub Vec<Entity>);

/// Component on waypoint entities that indicates which entity owns them.
/// This allows multiple players/AI entities to have their own waypoints.
#[derive(Component, Debug)]
pub struct WaypointOwner(pub Entity);

impl WaypointQueue {
    /// Get the next waypoint to follow (first in queue)
    pub fn next(&self) -> Option<Entity> {
        self.0.first().copied()
    }

    /// Remove the first waypoint from the queue
    pub fn advance(&mut self) -> Option<Entity> {
        if !self.0.is_empty() {
            Some(self.0.remove(0))
        } else {
            None
        }
    }

    /// Clear all waypoints
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Add a waypoint to the end of the queue
    pub fn push(&mut self, waypoint: Entity) {
        self.0.push(waypoint);
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
