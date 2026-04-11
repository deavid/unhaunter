use bevy::prelude::*;

/// Accumulator flag: set to `true` once all players are back in the truck.
///
/// Gated by the mission-end logic; the truck UI reads it to enable the "End Mission" button.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissionEndRequested(pub bool);

/// Inserted by the client when ServerGamePhase::Concluding is observed.
/// Tracks the local fade-to-black timer before transitioning to Summary.
#[derive(Resource)]
pub struct MissionConcludingCinematic {
    /// Countdown timer. When finished, the client transitions to AppState::Summary.
    ///
    /// SummaryData is local-only and is not a replication gate for this transition.
    pub timer: Timer,
    /// Whether player inputs have been disabled for the duration of this cinematic.
    pub inputs_blocked: bool,
}
