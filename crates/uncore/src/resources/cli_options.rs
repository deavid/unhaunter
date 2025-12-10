use bevy::prelude::Resource;

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct CliOptions {
    pub include_draft_maps: bool,
}
