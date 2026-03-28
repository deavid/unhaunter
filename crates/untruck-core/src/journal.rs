use bevy::prelude::*;
use uninvestigation_core::evidence::Evidence;

/// Event to force discard an evidence type in the journal UI.
#[derive(Message, Debug, Clone, Copy)]
pub struct ForceDiscardEvidenceEvent(pub Evidence);
