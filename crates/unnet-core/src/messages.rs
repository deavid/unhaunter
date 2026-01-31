use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerState {
    pub id: u64,
    pub position: [f32; 4], // x, y, z, orientation
                            // We can add animation states later
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GhostState {
    pub position: [f32; 3],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapTileState {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub tileset: String,
    pub tileuid: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomSync {
    pub name: String,
    pub state: u8, // 0=Off, 1=On
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GearSyncState {
    pub id: u32,
    pub position: [f32; 3],
    pub is_on: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    /// Initial handshake from Client to Host.
    Hello { version: String },
    /// Response from Host to Client.
    Welcome {
        id: u64,
        map_seed: u64,
        map_filepath: String,
        difficulty_id: String,
    },
    /// Periodic state update from Host to Client.
    Snapshot {
        tick: u64,
        game_state: String,
        players: Vec<PlayerState>,
        ghosts: Vec<GhostState>,
        rooms: Vec<RoomSync>,
        map_tiles: Vec<MapTileState>,
        gear: Vec<GearSyncState>,
    },
    /// Periodic input update from Client to Host.
    PlayerInput {
        movement: [f32; 2],
        run: bool,
        interact: bool,
        grab: bool,
        drop: bool,
        use_right_hand: bool,
        use_left_hand: bool,
        inventory_cycle: bool,
        inventory_swap: bool,
    },
}
