use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CliNetMode {
    #[default]
    Offline,
    PeerHost {
        port: u16,
        bind_addresses: Vec<String>,
    },
    Join {
        address: String,
        /// Base64 encoded connection ticket (postcard + HMAC-SHA256) issued by
        /// the Hub after PoW challenge. When present the Renet transport embeds
        /// it in the connection `user_data` so the dedicated server can validate
        /// the connection. `None` in singleplayer / direct-connect scenarios
        /// (no auth enforced).
        ticket: Option<String>,
    },
}

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CliOptions {
    pub include_draft_maps: bool,
    pub net_mode: CliNetMode,
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
        !matches!(self.net_mode, CliNetMode::Join { .. })
    }
}
