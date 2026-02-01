use crate::network_id::NetworkId;
use bevy::prelude::Message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerState {
    pub id: NetworkId,
    pub position: [f32; 4], // x, y, z, orientation
    pub is_hiding: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerGearState {
    pub player_id: NetworkId,
    pub left_hand: Option<NetworkId>,
    pub right_hand: Option<NetworkId>,
    pub inventory: Vec<NetworkId>,
    pub held_item: Option<NetworkId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GhostState {
    pub id: NetworkId,
    pub position: [f32; 3],
    pub warp: f32,
    pub hunt_warning_active: bool,
    pub hunt_warning_intensity: f32,
    pub calm_time_secs: f32,
    pub repellent_hits_delta: f32,
    pub repellent_misses_delta: f32,
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
    pub id: NetworkId,
    pub position: [f32; 3],
    pub is_on: bool,
    pub mode: Option<String>,
    pub battery: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    /// Initial handshake from Client to Host.
    Hello { version: String },
    /// Response from Host to Client.
    Welcome {
        id: NetworkId,
        map_seed: u64,
        map_filepath: String,
        difficulty_id: String,
    },
    /// Periodic state update from Host to Client.
    Snapshot {
        tick: u64,
        app_state: String,
        game_state: String,
        players: Vec<PlayerState>,
        ghosts: Vec<GhostState>,
        rooms: Vec<RoomSync>,
        map_tiles: Vec<MapTileState>,
        gear: Vec<GearSyncState>,
        player_gear: Vec<PlayerGearState>,
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
        target_position: Option<[f32; 2]>,
    },
    /// Client requests to enter the truck/van.
    RequestTruckEntry,
    /// Replication of a sound event from Host to Client.
    SoundEvent {
        sound_file: String,
        volume: f32,
        position: Option<[f32; 3]>,
    },
}

#[derive(Debug, Clone, Message)]
pub struct NetworkDataEvent {
    pub message: NetworkMessage,
}
