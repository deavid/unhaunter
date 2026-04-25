use dashmap::DashMap;
use moka::sync::Cache;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use unhub_client::protocol::{LibraryEntry, RoomSummary};
use uuid::Uuid;

#[derive(Clone)]
pub struct HubState {
    pub rooms: Arc<DashMap<String, RoomSummary>>,
    pub procmans: Arc<DashMap<Uuid, ProcManSession>>,
    pub config: Arc<tokio::sync::RwLock<HubConfig>>,
    pub start_time: std::time::Instant,
    pub rooms_by_ip: Arc<DashMap<std::net::IpAddr, Vec<String>>>,
    pub room_to_ip: Arc<DashMap<String, std::net::IpAddr>>,
    pub nonces: Arc<DashMap<String, NonceEntry>>,
    pub active_players: Cache<Uuid, ()>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HubConfig {
    pub version: u32,
    #[serde(default = "default_api_bind")]
    pub api_bind: String,
    #[serde(default = "default_procman_bind")]
    pub procman_bind: String,
    pub official_server_keys: std::collections::HashMap<Uuid, String>,
    pub banned_uuids: HashSet<Uuid>,
    pub allowed_procman_uuids: HashSet<Uuid>,
    pub max_rooms_per_ip: usize,
    pub trust_proxy_headers: bool,
    pub pow_difficulty: u32,
}

fn default_api_bind() -> String {
    "127.0.0.1:3000".to_string()
}

fn default_procman_bind() -> String {
    "127.0.0.1:11000".to_string()
}

pub struct NonceEntry {
    pub player_uuid: Uuid,
    pub client_ip: std::net::IpAddr,
    pub issued_at: std::time::Instant,
}

pub struct ProcManSession {
    pub tx: tokio::sync::mpsc::UnboundedSender<unhub_client::protocol::ProcManMessage>,
    pub library: Vec<LibraryEntry>,
    pub public_addr: String,
    pub idle_capacity: usize,
    pub last_heartbeat: std::time::Instant,
    /// HMAC-SHA256 key used to sign JWT connection tickets for this procman's
    /// dedicated servers. Sent during handshake and stored for ticket issuance.
    pub ticket_hmac_secret: String,
}

impl HubState {
    pub fn new(config: HubConfig) -> Self {
        Self {
            rooms: Arc::new(DashMap::new()),
            procmans: Arc::new(DashMap::new()),
            config: Arc::new(tokio::sync::RwLock::new(config)),
            start_time: std::time::Instant::now(),
            rooms_by_ip: Arc::new(DashMap::new()),
            room_to_ip: Arc::new(DashMap::new()),
            nonces: Arc::new(DashMap::new()),
            active_players: Cache::builder()
                .max_capacity(100_000)
                .time_to_live(Duration::from_secs(7200))
                .build(),
        }
    }
}
