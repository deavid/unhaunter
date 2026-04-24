use bevy::prelude::*;
use uninvestigation_core::ghost::GhostType;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;

/// Ghost-domain signal exported for downstream systems that need hunt-state context
/// without reading ghost-internal components directly.
#[derive(Debug, Clone, Copy)]
pub struct GhostHuntPressure {
    pub position: Position,
    pub calm_time_secs: f32,
    pub is_warping: bool,
}

/// Single-ghost mission snapshot exported for read-only consumers that need stable
/// ghost-domain facts without querying GhostSprite directly.
#[derive(Debug, Clone)]
pub struct PrimaryGhostSignal {
    pub class: GhostType,
    pub position: Position,
    pub spawn_point: BoardPosition,
    pub health: f32,
    pub hunting: f32,
    pub hunt_warning_active: bool,
    pub repellent_hits: i64,
}

/// Aggregated ghost-domain signals refreshed each frame by ghost logic.
#[derive(Resource, Debug, Clone, Default)]
pub struct GhostHuntSignals {
    pub any_present: bool,
    pub any_hunting: bool,
    pub any_warning_active: bool,
    pub any_hunted_this_mission: bool,
    pub any_hunt_likely: bool,
    pub any_near_hunt_without_warning: bool,
    pub primary: Option<PrimaryGhostSignal>,
    pub pressures: Vec<GhostHuntPressure>,
}
