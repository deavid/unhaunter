use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use unhub_client::protocol::RoomSummary;
use uuid::Uuid;

#[derive(Clone)]
pub struct HubState {
    pub rooms: Arc<DashMap<String, RoomSummary>>,
    pub procmans: Arc<DashMap<Uuid, ProcManSession>>,
    pub config: Arc<tokio::sync::RwLock<HubConfig>>,
    pub start_time: std::time::Instant,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HubConfig {
    pub version: u32,
    pub official_server_keys: std::collections::HashMap<Uuid, String>,
    pub banned_uuids: HashSet<Uuid>,
    pub allowed_procman_uuids: HashSet<Uuid>,
}

pub struct ProcManSession {
    pub tx: tokio::sync::mpsc::UnboundedSender<unhub_client::protocol::ProcManMessage>,
    pub game_versions: Vec<String>,
    pub public_addr: String,
    pub idle_capacity: usize,
    pub last_heartbeat: std::time::Instant,
}

impl HubState {
    pub fn new(config: HubConfig) -> Self {
        Self {
            rooms: Arc::new(DashMap::new()),
            procmans: Arc::new(DashMap::new()),
            config: Arc::new(tokio::sync::RwLock::new(config)),
            start_time: std::time::Instant::now(),
        }
    }
}
