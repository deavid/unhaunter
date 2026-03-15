use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Server-side lobby state replicated to all clients.
///
/// Spawned on a single "lobby state entity" when the server enters `AppState::Lobby`.
/// Clients query this component directly in UI systems (`Query<&LobbyInfo>`) to read
/// the current lobby roster, selected map, difficulty, and leader identity.
#[derive(Component, Debug, Default, Clone, Serialize, Deserialize)]
pub struct LobbyInfo {
    /// All players currently tracked in the lobby, in join order.
    pub players: Vec<LobbyPlayerInfo>,
    /// Path to the selected map TMX file, or `None` if not yet chosen.
    pub selected_map: Option<String>,
    /// String key of the selected difficulty (see `Difficulty::to_string`).
    pub selected_difficulty: String,
    /// UUID of the player currently holding lobby leader permissions.
    /// None = no leader (server boot, dedicated server before first player joins).
    pub leader_uuid: Option<Uuid>,
}

/// Per-player data stored inside `LobbyInfo`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyPlayerInfo {
    /// Stable persistent identity (from the player's profile installation_id).
    pub player_uuid: Uuid,
    /// The current active transport socket, or None if disconnected.
    #[serde(skip)]
    pub current_socket: Option<crate::ownership::OwnerId>,
    /// Index into the player-tint colour palette.
    pub tint_color_index: u8,
    /// `true` while the player's transport is still connected.
    pub connected: bool,
    /// Optional display name.
    pub nickname: Option<String>,
}

/// The authoritative game phase as determined by the server.
///
/// Replicated to all clients. Clients react to transitions in a context-aware,
/// guarded way (see `bridge.rs`) rather than blindly copying this into their
/// own `NextState<AppState>`.
///
/// This is **not** an `AppState`. It describes what the server's game session
/// is doing, not how any client's UI is laid out.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerGamePhase {
    /// Server is up and accepting players. No mission is running.
    Lobby,
    /// A mission is in progress.
    InProgress,
    /// Mission over; server is computing/publishing results. All ticking has stopped.
    Concluding,
    /// Results published and available on SummaryData.
    Ended,
}

/// Spawned (with `Replicated`) by the server when a mission is about to start.
///
/// Clients observe `On<Add, SelectedMission>` to fire `LoadLevelEvent` and transition
/// their own state to `AppState::MissionLoading`.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SelectedMission {
    /// Path to the TMX map file.
    pub map_path: String,
    /// Seed used for procedural generation in this mission run.
    pub map_seed: u64,
    /// String key of the difficulty chosen for this mission.
    pub difficulty_id: String,
    /// Server-side unix timestamp (seconds) when this mission was started.
    #[serde(default)]
    pub started_at_unix_secs: f64,
}

/// Marker resource: inserted by `unreplicon-plugin` when replicon-based player
/// spawning is active (i.e. the server entered `AppState::InGame`).
///
/// When this resource is present, classic-mode player-spawning systems such as
/// `spawn_joined_player` should be skipped to avoid creating duplicate entities.
#[derive(Resource, Debug, Default)]
pub struct RepliconPlayerSpawningActive;

/// Marker resource: inserted by `unreplicon-plugin` when replicon-based ghost
/// spawning is active (i.e. the server entered `AppState::InGame`).
///
/// When this resource is present, `classic_mode_orchestrator` in
/// `unclassic-mode-plugin` skips the ghost-spawn block so that the ghost entity
/// is only created on the server and then replicated to Join clients.
#[derive(Resource, Debug, Default)]
pub struct RepliconGhostSpawningActive;

/// Marker component placed on the singleton "mission goal" entity.
///
/// This entity carries replicated journal components (`GhostGuess`,
/// `SummaryData`) so clients can receive them from the server without the
/// need for a per-ghost or per-player lookup.
#[derive(Component, Debug, Default, Clone, Serialize, Deserialize)]
pub struct MissionGoalEntity;
