use bevy::prelude::*;
use uncore_foundation::types::evidence::Evidence;

/// Event to force discard an evidence type in the journal UI.
#[derive(Message, Debug, Clone, Copy)]
pub struct ForceDiscardEvidenceEvent(pub Evidence);
