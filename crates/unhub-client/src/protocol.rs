use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Hub ↔ ProcMan Protocol ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ProcManMessage {
    // ProcMan → Hub
    ProcManHello {
        uuid: Uuid,
        version: String,
        game_versions: Vec<String>,
        port_range: (u16, u16),
        public_addr: String,
        idle_pool: std::collections::HashMap<String, usize>,
        rooms: Vec<RoomSummary>,
    },
    Heartbeat {
        idle_capacity: usize,
        rooms: Vec<RoomSummary>,
    },
    RoomReady {
        room: RoomSummary,
    },
    CreateRoomFailed {
        room_code: String,
        reason: String,
    },
    PlayerJoined {
        room_code: String,
        player_uuid: Uuid,
        new_player_count: u8,
    },
    PlayerLeft {
        room_code: String,
        player_uuid: Uuid,
        remaining_count: u8,
    },
    RoomStateChanged {
        room_code: String,
        old_state: RoomState,
        new_state: RoomState,
        metadata: RoomMetadata,
    },
    RoomClosed {
        room_code: String,
        port: u16,
        reason: String,
    },

    // Hub → ProcMan
    ProcManAccepted {
        hub_version: String,
    },
    AuthRejected {
        reason: String,
    },
    CreateRoom {
        room_code: String,
        secret: String,
        game_version: String,
    },
    KillRoom {
        room_code: String,
        reason: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomState {
    Lobby,
    InGame,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomMetadata {
    pub map: String,
    pub difficulty: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomSummary {
    pub code: String,
    pub port: u16,
    pub game_version: String,
    pub secret: String,
    pub state: RoomState,
    pub player_count: u8,
    pub metadata: RoomMetadata,
    pub server_id: Uuid,
}

// --- Player ↔ Hub REST API ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateRoomRequest {
    pub player_uuid: Uuid,
    pub game_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinRoomRequest {
    pub player_uuid: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateRoomResponse {
    pub code: String,
    pub addr: String,
    pub secret: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinRoomResponse {
    pub code: String,
    pub addr: String,
    pub secret: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HubError {
    pub error: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HealthResponse {
    pub hub_version: String,
    pub uptime_seconds: u64,
}

// --- ProcMan ↔ Dedicated Protocol ---

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ProcManToDedicated {
    AssignRoom {
        room_code: String,
        secret: String,
    },
    RenameRoom {
        new_code: String,
        new_secret: String,
    },
    /// Wipe current room assignment and return to idle state.
    WipeRoom {
        reason: String,
    },
    Shutdown {
        reason: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum DedicatedToProcMan {
    Ready {
        port: u16,
    },
    PlayerJoined {
        player_uuid: Uuid,
    },
    PlayerLeft {
        player_uuid: Uuid,
        remaining_count: usize,
    },
    StateChanged {
        state: RoomState,
        metadata: RoomMetadata,
    },
    RoomRenameRequest {
        reason: String,
    },
    Exiting {
        reason: String,
    },
}
