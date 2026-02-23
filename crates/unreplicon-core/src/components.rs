use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use untypes_core::states::AppState;

/// Server-side lobby state replicated to all clients.
///
/// Spawned on a single "lobby state entity" when the server enters `AppState::Lobby`.
/// Clients read this via the bridge system to populate `LobbyData`.
#[derive(Component, Debug, Default, Clone, Serialize, Deserialize)]
pub struct LobbyInfo {
    /// All players currently tracked in the lobby, in join order.
    pub players: Vec<LobbyPlayerInfo>,
    /// Path to the selected map TMX file, or `None` if not yet chosen.
    pub selected_map: Option<String>,
    /// String key of the selected difficulty (see `Difficulty::to_string`).
    pub selected_difficulty: String,
    /// `NetworkId` u64 of the player who owns (controls) the lobby.
    /// `0` means the server itself (Host/listen-server mode).
    pub owner_client_id: u64,
}

/// Per-player data stored inside `LobbyInfo`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyPlayerInfo {
    /// renet client id (u64) for this player. `0` = server/host.
    pub client_id: u64,
    /// Index into the player-tint colour palette.
    pub tint_color_index: u8,
    /// `true` while the player's transport is still connected.
    pub connected: bool,
    /// Optional display name.
    pub nickname: Option<String>,
}

/// Replicated wrapper around the server's current `AppState`.
///
/// Clients watch this component changing and drive their own `NextState<AppState>`.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ServerAppState(pub AppState);

/// Spawned (with `Replicated`) by the server when a mission is about to start.
///
/// Clients observe `On<Add, SelectedMission>` to fire `LoadLevelEvent` and transition
/// their own state to `AppState::Loading`.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SelectedMission {
    /// Path to the TMX map file.
    pub map_path: String,
    /// Seed used for procedural generation in this mission run.
    pub map_seed: u64,
    /// String key of the difficulty chosen for this mission.
    pub difficulty_id: String,
}
