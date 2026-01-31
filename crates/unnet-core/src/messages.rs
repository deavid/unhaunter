use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerState {
    pub id: u64,
    pub position: [f32; 4], // x, y, z, orientation
                            // We can add animation states later
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    /// Initial handshake from Client to Host.
    Hello { version: String },
    /// Response from Host to Client.
    Welcome { id: u64, map_seed: u64 },
    /// Periodic state update from Host to Client.
    Snapshot {
        tick: u64,
        players: Vec<PlayerState>,
    },
    /// Periodic input update from Client to Host.
    PlayerInput {
        movement: [f32; 2],
        run: bool,
        interact: bool,
    },
}
