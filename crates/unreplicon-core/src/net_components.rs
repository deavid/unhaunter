use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use ungearitems_core::components::flashlight::FlashlightStatus;
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
