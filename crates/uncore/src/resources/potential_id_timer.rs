use crate::types::evidence::Evidence;
use crate::types::ghost::types::GhostType;
use bevy::prelude::Resource;

#[derive(Resource, Default, Debug)]
pub struct PotentialIDTimer {
    /// Stores information about a potential ghost identification.
    pub data: Option<PotentialIDData>,
}

/// Represents data associated with a potential ghost identification.
#[derive(Debug)]
pub struct PotentialIDData {
    /// Newly detected high-clarity evidence.
    pub evidence: Evidence,
    /// The single potential ghost identified.
    pub ghost_type: GhostType,
    /// Initial acknowledgement count for this evidence on gear.
    pub ack_count: u32,
    /// Time when this potential ID was first detected.
    pub detection_time: f32,
}
