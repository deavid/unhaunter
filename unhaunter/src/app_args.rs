use bevy::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
        /// the Hub after PoW challenge. When present the transport sends it
        /// in a post-connection handshake message so the dedicated server
        /// can validate the connection. `None` in singleplayer / direct-connect
        /// scenarios (no auth enforced).
        ticket: Option<String>,
    },
}

impl CliNetMode {
    pub fn is_authority(&self) -> bool {
        !matches!(self, CliNetMode::Join { .. })
    }
}

/// Binary-internal app arguments. Fields here are consumed at app-setup time
/// and never inserted into the ECS world.
pub struct AppArgs {
    /// Verbosity level for the log filter (0 = default, higher = more verbose).
    pub verbose: u8,
    /// When `true`, audio is silenced at startup.
    pub mute: bool,
    // --- fields forwarded into CliOptions ---
    pub include_draft_maps: bool,
    pub net_mode: CliNetMode,
    pub installation_id_file: Option<String>,
    pub dedicated: bool,
    pub procman_channel: Option<String>,
    pub hub_url: Option<String>,
    pub cert_file: Option<String>,
    pub key_file: Option<String>,
    pub skip_ssl_verification: bool,
}

impl AppArgs {
    pub fn default_wasm() -> Self {
        Self {
            verbose: 0,
            mute: false,
            include_draft_maps: false,
            net_mode: CliNetMode::Offline,
            installation_id_file: None,
            dedicated: false,
            procman_channel: None,
            hub_url: None,
            cert_file: None,
            key_file: None,
            skip_ssl_verification: false,
        }
    }
}
