use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use unhub_client::protocol::{DedicatedToProcMan, ProcManToDedicated};

/// Bidirectional channel to the process manager over stdin/stdout.
///
/// Only present when `CliOptions::procman_channel == Some("stdin")`.
#[derive(Resource)]
pub struct ProcManChannel {
    /// Sender for outgoing messages to procman (player events, state sync).
    /// Used in Phase 2+ systems that report game state back to procman.
    #[allow(dead_code)]
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
