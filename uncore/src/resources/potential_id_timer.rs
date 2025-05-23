use crate::types::evidence::Evidence;
use crate::types::ghost::types::GhostType;
use bevy::prelude::Resource;

#[derive(Resource, Default, Debug)]
pub struct PotentialIDTimer {
    // Stores: (Newly detected high-clarity evidence,
    //          The single potential ghost identified,
    //          Initial acknowledgement count for this evidence on gear,
    //          Time when this potential ID was first detected)
    pub data: Option<(Evidence, GhostType, u32, f32)>,
}
