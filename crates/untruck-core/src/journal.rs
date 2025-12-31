use bevy::prelude::*;
use unfoundation_core::types::evidence::Evidence;

/// Event to force discard an evidence type in the journal UI.
#[derive(Message, Debug, Clone, Copy)]
pub struct ForceDiscardEvidenceEvent(pub Evidence);
