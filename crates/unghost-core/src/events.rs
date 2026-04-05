use bevy::prelude::*;
use uninvestigation_core::evidence::Evidence;
use uninvestigation_core::ghost::GhostType;
use unspatial_core::position::Position;

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

/// Emitted by the networking layer when a client requests a journal evidence toggle.
/// `unghost-plugin` reacts and applies the change to `GhostGuess`.
#[derive(Message, Debug, Clone)]
pub struct JournalEvidenceToggled {
    pub evidence: Evidence,
    pub discard: bool,
    pub mark_as_found: bool,
}

/// Emitted by the networking layer when a client requests a journal ghost-type toggle.
/// `unghost-plugin` reacts and applies the change to `GhostGuess`.
#[derive(Message, Debug, Clone)]
pub struct JournalGhostToggled {
    pub ghost_type: Option<GhostType>,
    pub discard: bool,
}

/// Emitted by `unghost-plugin` when evidence clarity crosses the high-clarity threshold.
/// `untruck-plugin` consumes this to update its local blinking state without polling
/// `CurrentEvidenceReadings` directly.
#[derive(Message, Debug, Clone)]
pub struct EvidenceClarityThresholdCrossed {
    pub evidence: Evidence,
    /// `true` when clarity crossed *above* the threshold, `false` when crossing below.
    pub above_threshold: bool,
}

/// Emitted by `unghost-plugin` when the ghost's actual `GhostType` becomes known or changes.
/// `untruck-plugin` caches this to avoid querying `GhostSprite` directly.
#[derive(Message, Debug, Clone)]
pub struct GhostActualTypeChanged {
    pub ghost_type: GhostType,
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

/// Emitted by ghost logic when a breaker trip should materialize local spark particles.
#[derive(Message, Debug, Clone)]
pub struct GhostBreakerSparkRequest {
    pub position: Position,
}
