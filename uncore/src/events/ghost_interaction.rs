use crate::components::board::position::Position;
use bevy::prelude::*;

/// Represents the type of interaction a ghost can perform with the environment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhostInteractionType {
    /// Toggle lights or switches on/off
    Toggle,
    /// Slam a door shut quickly with loud noise
    DoorSlam,
    /// Slowly creak a door open or closed
    DoorCreak,
    /// Throw a light object across the room
    Throw,
    /// Slightly nudge an object
    Nudge,
    /// Slowly slide an object to a new position
    HauntedMove,
    /// Temporarily lock a door
    Lock,
    /// Trip the main circuit breaker
    TripBreaker,
}

/// Event dispatched when a ghost performs an interaction with an environmental object
#[derive(Message, Debug, Clone)]
pub struct GhostInteractionEvent {
    /// The entity that the ghost is interacting with
    pub target: Entity,
    /// The type of interaction being performed
    pub interaction_type: GhostInteractionType,
    /// Optional destination position for movement-based interactions (Throw, HauntedMove)
    pub destination: Option<Position>,
}
