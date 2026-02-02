use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetMode {
    #[default]
    Offline,
    Host {
        port: u16,
    },
    Join {
        address: String,
    },
}

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CliOptions {
    pub include_draft_maps: bool,
    pub net_mode: NetMode,
    pub map_path: Option<String>,
    pub difficulty_id: Option<String>,
    pub verbose: u8,
}

pub fn is_host(cli: Res<CliOptions>) -> bool {
    !matches!(cli.net_mode, NetMode::Join { .. })
}

pub fn is_client(cli: Res<CliOptions>) -> bool {
    matches!(cli.net_mode, NetMode::Join { .. })
}
