use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::evidence::Evidence;
use crate::ghost::GhostType;

/// Sent by a client to toggle evidence in the shared journal.
///
/// The server validates this, updates `GhostGuess`, and the change propagates
/// to all clients via replication.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct RequestJournalEvidenceToggle {
    pub evidence: Evidence,
    pub discard: bool,
    pub mark_as_found: bool,
}

/// Sent by a client to update the ghost-type guess in the shared journal.
///
/// `ghost_type = None` means "clear the current guess".
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct RequestJournalGhostToggle {
    pub discard: bool,
    pub ghost_type: Option<GhostType>,
}
