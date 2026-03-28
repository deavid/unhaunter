use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Default)]
pub struct TmxMapConfig {
    pub include_draft_maps: bool,
}
