use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use unhub_client::protocol::{DedicatedToProcMan, ProcManToDedicated};

#[derive(Resource, Debug, Clone)]
pub enum TransportConfig {
    Offline,
    PeerHost {
        port: u16,
        cert_file: Option<String>,
        key_file: Option<String>,
        skip_ssl_verification: bool,
    },
    Join {
        address: String,
        /// TLS SNI hostname to present during the QUIC handshake. When set,
        /// Quinnet validates the server certificate against this name instead
        /// of the raw IP in `address`.
        server_hostname: Option<String>,
        ticket: Option<String>,
        skip_ssl_verification: bool,
    },
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ProcManConfig {
    pub procman_channel: Option<String>,
    pub port: u16,
    pub cert_file: Option<String>,
    pub key_file: Option<String>,
    pub skip_ssl_verification: bool,
}

/// Bidirectional channel to the process manager over stdin/stdout.
#[derive(Resource)]
pub struct ProcManChannel {
    /// Sender for outgoing messages to procman (player events, state sync).
    /// Used in Phase 2+ systems that report game state back to procman.
    pub tx: Sender<DedicatedToProcMan>,
    pub rx: Receiver<ProcManToDedicated>,
}

/// Authentication state received from the process manager via `AssignRoom`.
///
/// Populated once procman assigns a room to this dedicated server. Before
/// that point (when both fields are `None`) all incoming Renet connections are
/// rejected.
#[derive(Resource, Default)]
pub struct RoomAuth {
    /// The room code this server is currently hosting, or `None` if idle.
    pub room_code: Option<String>,
    /// HMAC-SHA256 key (hex-encoded 32 bytes) for validating JWT tickets.
    pub ticket_hmac_secret: Option<String>,
}
