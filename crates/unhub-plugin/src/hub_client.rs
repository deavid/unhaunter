use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use bevy::tasks::AsyncComputeTaskPool;
use bevy_replicon::prelude::ProtocolHash;
use crossbeam_channel::{Receiver, Sender};
use std::future::Future;
use unhub_client::protocol::{
    ChallengeRequest, ChallengeResponse, CreateRoomRequest, CreateRoomResponse, JoinRoomRequest,
    JoinRoomResponse, MultiplayerStatus, PingRequest, PingResponse,
};

#[derive(Resource, Debug, Clone, Default)]
pub struct HubConfig {
    pub hub_url: Option<String>,
}

#[derive(Resource)]
pub struct HubClient {
    hub_url: String,
    tx: Sender<HubResponse>,
    pub rx: Receiver<HubResponse>,
    pub session_id: u16,
}

#[cfg(target_arch = "wasm32")]
fn spawn_hub_task<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    AsyncComputeTaskPool::get().spawn(future).detach();
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_hub_task<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    static HUB_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
        std::sync::LazyLock::new(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .expect("Failed to create Unhaunter Hub Runtime")
        });

    std::mem::drop(HUB_RUNTIME.spawn(future));
}

impl HubClient {
    pub fn create_room(&self, player_uuid: uuid::Uuid, game_version: String, protocol_hash: u64) {
        let hub_url = self.hub_url.clone();
        let tx = self.tx.clone();
        spawn_hub_task(async move {
            let client = reqwest::Client::new();
            // 1. Request Challenge
            let challenge_res = client
                .post(format!("{}/v1/challenge", hub_url))
                .json(&ChallengeRequest { player_uuid })
                .send()
                .await;
            let resp = match challenge_res {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(HubResponse::Error(format!(
                        "Failed to request challenge: {}",
                        e
                    )));
                    return;
                }
            };
            if !resp.status().is_success() {
                let _ = tx.send(HubResponse::Error(format!(
                    "Challenge failed: {}",
                    resp.status()
                )));
                return;
            }
            let Ok(challenge) = resp.json::<ChallengeResponse>().await else {
                let _ = tx.send(HubResponse::Error(
                    "Failed to parse challenge response".to_string(),
                ));
                return;
            };
            // 2. Solve PoW
            #[cfg(target_arch = "wasm32")]
            let solution =
                unhub_client::solve_pow_async(&challenge.nonce, challenge.difficulty).await;
            #[cfg(not(target_arch = "wasm32"))]
            let solution = unhub_client::solve_pow(&challenge.nonce, challenge.difficulty);

            // 3. Create Room
            match client
                .post(format!("{}/v1/rooms/create", hub_url))
                .json(&CreateRoomRequest {
                    player_uuid,
                    game_version,
                    protocol_hash,
                    nonce: challenge.nonce,
                    solution,
                })
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    let status = resp.status();
                    match resp.text().await {
                        Ok(body) => match serde_json::from_str::<CreateRoomResponse>(&body) {
                            Ok(data) => {
                                let _ = tx.send(HubResponse::RoomCreated(data));
                            }
                            Err(e) => {
                                let _ = tx.send(HubResponse::Error(format!(
                                        "Failed to parse create room response (status: {}): {}; body: {}",
                                        status, e, body
                                    )));
                            }
                        },
                        Err(e) => {
                            let _ = tx.send(HubResponse::Error(format!(
                                "Failed to read create room response body (status: {}): {}",
                                status, e
                            )));
                        }
                    }
                }
                Ok(resp) => {
                    let _ = tx.send(HubResponse::Error(format!("Status: {}", resp.status())));
                }
                Err(e) => {
                    let _ = tx.send(HubResponse::Error(e.to_string()));
                }
            }
        });
    }

    pub fn join_room(&self, code: String, player_uuid: uuid::Uuid) {
        let hub_url = self.hub_url.clone();
        let tx = self.tx.clone();
        spawn_hub_task(async move {
            let client = reqwest::Client::new();
            match client
                .post(format!("{}/v1/rooms/join/{}", hub_url, code))
                .json(&JoinRoomRequest { player_uuid })
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    let status = resp.status();
                    match resp.text().await {
                        Ok(body) => match serde_json::from_str::<JoinRoomResponse>(&body) {
                            Ok(data) => {
                                let _ = tx.send(HubResponse::RoomJoined(data));
                            }
                            Err(e) => {
                                let _ = tx.send(HubResponse::Error(format!(
                                    "Failed to parse join room response (status: {}): {}; body: {}",
                                    status, e, body
                                )));
                            }
                        },
                        Err(e) => {
                            let _ = tx.send(HubResponse::Error(format!(
                                "Failed to read join room response body (status: {}): {}",
                                status, e
                            )));
                        }
                    }
                }
                Ok(resp) => {
                    let _ = tx.send(HubResponse::Error(format!("Status: {}", resp.status())));
                }
                Err(e) => {
                    let _ = tx.send(HubResponse::Error(e.to_string()));
                }
            }
        });
    }

    pub fn ping(
        &self,
        installation_id: uuid::Uuid,
        session_id: u16,
        version: String,
        protocol_hash: u64,
    ) {
        let hub_url = self.hub_url.clone();
        let tx = self.tx.clone();
        spawn_hub_task(async move {
            let client = reqwest::Client::new();
            match client
                .post(format!("{}/v1/ping", hub_url))
                .json(&PingRequest {
                    installation_id,
                    session_id,
                    version,
                    protocol_hash,
                })
                .timeout(std::time::Duration::from_secs(3))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<PingResponse>().await {
                        let _ = tx.send(HubResponse::PingResult {
                            ok: data.ok,
                            online_players: data.online_players_estimate,
                            multiplayer_status: data.multiplayer_status,
                            upgrade_version: data.upgrade_version,
                        });
                    } else {
                        let _ = tx.send(HubResponse::PingResult {
                            ok: false,
                            online_players: 0,
                            multiplayer_status: MultiplayerStatus::Unsupported,
                            upgrade_version: None,
                        });
                    }
                }
                _ => {
                    let _ = tx.send(HubResponse::PingResult {
                        ok: false,
                        online_players: 0,
                        multiplayer_status: MultiplayerStatus::Unsupported,
                        upgrade_version: None,
                    });
                }
            }
        });
    }
}

pub enum HubResponse {
    RoomCreated(CreateRoomResponse),
    RoomJoined(JoinRoomResponse),
    PingResult {
        ok: bool,
        online_players: usize,
        multiplayer_status: MultiplayerStatus,
        upgrade_version: Option<String>,
    },
    Error(String),
}

#[derive(Resource, Default)]
pub struct HubStatus {
    pub last_response: Option<HubResponse>,
    pub is_pending: bool,
    /// Set after hub responds with a room address; stays true while we wait for
    /// the server's LobbyInfo to arrive via replication before entering Lobby.
    pub is_connecting: bool,
    /// Counts down (in seconds) after LobbyInfo arrives before entering Lobby.
    /// None = not yet triggered; Some(t) = t seconds remaining.
    pub lobby_ready_timer: Option<f32>,
    pub is_online: bool,
    pub online_players: usize,
}

#[derive(Resource)]
pub struct HubPingTimer(pub Timer);

#[derive(Resource, Default)]
pub struct ClientProtocolHash(pub u64);

#[derive(Resource, Default, Clone, Debug)]
pub struct HubConnectionStatus {
    pub status: Option<MultiplayerStatus>,
    pub upgrade_version: Option<String>,
    pub last_update: f32,
}

impl Default for HubPingTimer {
    fn default() -> Self {
        let mut timer = Timer::new(std::time::Duration::from_secs(300), TimerMode::Repeating);
        // Force an immediate ping on the first run
        timer.tick(std::time::Duration::from_secs(300));
        Self(timer)
    }
}

pub fn setup_hub_client(mut commands: Commands, hub_config: Res<HubConfig>) {
    let (tx_to_bevy, rx_from_worker) = crossbeam_channel::unbounded::<HubResponse>();

    let hub_url = if let Some(url) = &hub_config.hub_url {
        url.clone()
    } else {
        info!("Hub URL not configured, using default https://hub.unhaunter.com");
        "https://hub.unhaunter.com".to_string()
    };

    let session_id = rand::random::<u16>();
    commands.insert_resource(HubClient {
        hub_url,
        tx: tx_to_bevy,
        rx: rx_from_worker,
        session_id,
    });
    commands.insert_resource(HubStatus::default());
    commands.insert_resource(HubPingTimer::default());
    commands.insert_resource(ClientProtocolHash::default());
    commands.insert_resource(HubConnectionStatus::default());
}

pub fn trigger_ping_system(mut timer: ResMut<HubPingTimer>) {
    // Setting ticked time to duration forces it to trigger immediately next Update
    timer.0.set_elapsed(std::time::Duration::from_secs(300));
}

pub fn ping_hub_system(
    time: Res<Time>,
    mut timer: ResMut<HubPingTimer>,
    client: Res<HubClient>,
    protocol_hash: Res<ClientProtocolHash>,
    profile: Option<Res<bevy_persistent::Persistent<unprofile_core::profile::PlayerProfileData>>>,
) {
    timer.0.tick(time.delta());

    if timer.0.just_finished()
        && let Some(profile) = &profile
    {
        client.ping(
            profile.installation_id,
            client.session_id,
            env!("CARGO_PKG_VERSION").to_string(),
            protocol_hash.0,
        );
    }
}

pub fn update_hub_status(
    mut status: ResMut<HubStatus>,
    mut hub_connection_status: ResMut<HubConnectionStatus>,
    client: Res<HubClient>,
) {
    while let Ok(resp) = client.rx.try_recv() {
        if let HubResponse::PingResult {
            ok,
            online_players,
            multiplayer_status,
            upgrade_version,
        } = resp
        {
            status.is_online = ok;
            status.online_players = online_players;
            hub_connection_status.status = Some(multiplayer_status);
            hub_connection_status.upgrade_version = upgrade_version;
            hub_connection_status.last_update = 0.0;
        } else {
            status.last_response = Some(resp);
            status.is_pending = false;
        }
    }
}

pub fn extract_protocol_hash(
    bevy_replicon_hash: Res<ProtocolHash>,
    mut client_hash: ResMut<ClientProtocolHash>,
) {
    // Deserialize ProtocolHash to get the numeric u64 value
    if let Ok(hash_str) = serde_json::to_string(&*bevy_replicon_hash) {
        if let Ok(hash_val) = hash_str.trim_matches('"').parse::<u64>() {
            client_hash.0 = hash_val;
            info!("Extracted protocol hash: {}", hash_val);
        } else {
            warn!(
                "Failed to parse protocol hash from bevy_replicon: {}",
                hash_str
            );
        }
    } else {
        warn!("Failed to serialize bevy_replicon ProtocolHash");
    }
}
