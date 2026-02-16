use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetMode {
    #[default]
    Offline,
    Host {
        port: u16,
        bind_addresses: Vec<String>,
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
    pub installation_id_file: Option<String>,
    pub verbose: u8,
    pub mute: bool,
    pub dedicated: bool,
    pub procman_channel: Option<String>,
    pub hub_url: Option<String>,
}

impl CliOptions {
    pub fn is_headless(&self) -> bool {
        self.dedicated
    }
    pub fn is_authority(&self) -> bool {
        !matches!(self.net_mode, NetMode::Join { .. })
    }
}

pub fn is_authority(cli: Res<CliOptions>) -> bool {
    cli.is_authority()
}

pub fn is_headless(cli: Res<CliOptions>) -> bool {
    cli.is_headless()
}

pub fn is_client(cli: Res<CliOptions>) -> bool {
    matches!(cli.net_mode, NetMode::Join { .. })
}
