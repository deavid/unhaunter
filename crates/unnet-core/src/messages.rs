use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    /// Initial handshake from Client to Host.
    Hello { version: String },
    /// Response from Host to Client.
    Welcome { id: u64, map_seed: u64 },
    /// Periodic state update from Host to Client.
    Snapshot {
        tick: u64,
        // Detailed state will move here later
    },
    /// Periodic input update from Client to Host.
    PlayerInput { movement: [f32; 2], interact: bool },
}
