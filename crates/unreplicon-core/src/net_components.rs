use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use ungearitems_core::components::flashlight::FlashlightStatus;
use unghost_core::types::evidence::Evidence;
use unghost_core::types::ghost::types::GhostType;

/// Replicated network position for a player.
///
/// Separated from `Position` (local, immediate) so remote players can be
/// interpolated smoothly without overwriting the local player's zero-latency position.
/// The server sets this from `handle_player_move` messages (for remote clients) or
/// directly from the local player entity's `Position` (for the host).
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Replicated player state flags and vital statistics.
///
/// Written by the server from client `PlayerMoveMessage` (remote clients) or by
/// `sync_player_state_to_net` (host player). Read by all clients to update remote
/// player visuals (hiding alpha, spectating state, etc.).
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerStateNet {
    pub is_hiding: bool,
    pub is_in_truck: bool,
    pub is_spectating: bool,
    pub stamina: f32,
    pub health: f32,
    pub sanity: f32,
    pub is_running: bool,
    /// Current animation frame index.
    pub frame: u16,
}

/// Replicated player identity for tint resolution and local-player detection.
///
/// Allows clients to determine which entity represents themselves (by matching
/// `client_id` against `LocalPlayer`) and to look up the correct tint colour
/// in the player colour palette.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerNetInfo {
    /// Replicon NetworkId for the owning client (0 = server/host sentinel).
    pub client_id: u64,
    /// Index into the player-tint colour palette.
    pub tint_color_index: u8,
}

// ---------------------------------------------------------------------------
// Gear net components — replicated, server-authoritative
// ---------------------------------------------------------------------------

/// Replicated flashlight state.
///
/// Driven by the server; clients apply changes to their local `Flashlight` component
/// via `On<Insert, FlashlightNet>` and `On<Changed, FlashlightNet>` observers
/// (see `unreplicon-plugin`).
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlashlightNet {
    pub status: FlashlightStatus,
    pub battery: f32,
}

/// Replicated thermometer state (deployed or stowed).
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThermometerNet {
    pub is_deployed: bool,
}

/// Replicated EMF meter state.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct EMFMeterNet {
    pub is_deployed: bool,
    pub is_on: bool,
}

/// Replicated spirit box state.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpiritBoxNet {
    pub is_on: bool,
    pub charge: f32,
}

/// Replicated sage bundle state.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct SageBundleNet {
    pub consumed: bool,
    pub is_active: bool,
    pub remaining_secs: f32,
}

/// Replicated repellent flask state.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepellentFlaskNet {
    pub qty: i32,
    pub active: bool,
    pub liquid_content: Option<GhostType>,
}

// ---------------------------------------------------------------------------
// Phase 4: Ghost, Evidence, and Mission net components
// ---------------------------------------------------------------------------

/// Replicated ghost AI state — server-authoritative, sent to all clients.
///
/// The server writes this every frame from `GhostSprite` + `GhostBehaviorDynamics`.
/// Clients apply it to their local `GhostSprite` via an observer.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct GhostStateNet {
    /// `GhostSprite.hunting > 0.1`
    pub is_hunting: bool,
    pub hunt_warning_active: bool,
    pub hunt_warning_intensity: f32,
    pub hunt_target: bool,
    /// From `GhostBehaviorDynamics.visual_alpha_multiplier`
    pub visual_alpha_multiplier: f32,
}

/// Replicated journal evidence state — placed on the `MissionGoalEntity`.
///
/// The server writes this from `GhostGuess` every frame.
/// Clients apply it to their local `GhostGuess` via an observer.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceFoundNet {
    pub found: Vec<Evidence>,
    pub missing: Vec<Evidence>,
    pub ghost_type_guess: Option<GhostType>,
    pub ghosts_discarded: Vec<GhostType>,
}

/// Replicated mission result — placed on the `MissionGoalEntity`.
///
/// Populated by the server when the mission ends.  Clients observe
/// `On<Add, MissionResultNet>` to populate their local `SummaryData`.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct MissionResultNet {
    pub time_taken_secs: f32,
    pub ghost_types: Vec<GhostType>,
    pub repellent_used_amt: u32,
    pub ghosts_unhaunted: u32,
    pub base_score: i64,
    pub difficulty_multiplier: f32,
    pub grade_multiplier: f64,
    pub average_sanity: f32,
    pub player_count: u32,
    pub alive_count: u32,
    pub full_score: i64,
    pub map_path: String,
    pub mission_successful: bool,
    pub money_earned: i64,
    /// String representation of `Grade`: "A", "B", "C", "D", "F", "NA".
    pub grade_achieved: String,
    pub required_deposit: i64,
    pub mission_reward_base: i64,
    pub deposit_originally_held: i64,
    pub deposit_returned_to_bank: i64,
    pub costs_deducted_from_deposit: i64,
    /// Becomes `true` once the server has filled in all result fields.
    pub ready: bool,
}
