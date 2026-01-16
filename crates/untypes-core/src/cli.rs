use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct CliOptions {
    pub include_draft_maps: bool,
}
