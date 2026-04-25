use bevy::prelude::*;
use bevy_replicon::prelude::ProtocolHash;
use crossbeam_channel::{Receiver, Sender};
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
    pub tx: Sender<HubRequest>,
    pub rx: Receiver<HubResponse>,
}

pub enum HubRequest {
    CreateRoom {
        player_uuid: uuid::Uuid,
        game_version: String,
        protocol_hash: u64,
    },
    JoinRoom {
        code: String,
        player_uuid: uuid::Uuid,
    },
    Ping {
        installation_id: uuid::Uuid,
        version: String,
        protocol_hash: u64,
    },
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
    let (tx_to_worker, rx_from_bevy) = crossbeam_channel::unbounded::<HubRequest>();
    let (tx_to_bevy, rx_from_worker) = crossbeam_channel::unbounded::<HubResponse>();

    let hub_url = if let Some(url) = &hub_config.hub_url {
        url.clone()
    } else {
        info!("Hub URL not configured, using default https://hub.unhaunter.com");
        "https://hub.unhaunter.com".to_string()
    };

    let worker_hub_url = hub_url.clone();
    std::thread::spawn(move || {
        // Intentionally single-threaded: Hub requests are sequential (one at a
        // time) and low-frequency. Multi-threading would add complexity and CPU
        // overhead for no practical gain in this use case.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let client = reqwest::Client::new();
            while let Ok(req) = rx_from_bevy.recv() {
                match req {
                    HubRequest::CreateRoom {
                        player_uuid,
                        game_version,
                        protocol_hash,
                    } => {
                        // 1. Request Challenge
                        let challenge_res = client
                            .post(format!("{}/v1/challenge", worker_hub_url))
                            .json(&ChallengeRequest { player_uuid })
                            .send()
                            .await;

                        if let Ok(resp) = challenge_res {
                            if resp.status().is_success() {
                                if let Ok(challenge) = resp.json::<ChallengeResponse>().await {
                                    // 2. Solve PoW
                                    let nonce = challenge.nonce.clone();
                                    let difficulty = challenge.difficulty;
                                    let solution = tokio::task::spawn_blocking(move || {
                                        unhub_client::solve_pow(&nonce, difficulty)
                                    })
                                    .await
                                    .unwrap_or_default();

                                    // 3. Create Room
                                    let create_res = client
                                        .post(format!("{}/v1/rooms/create", worker_hub_url))
                                        .json(&CreateRoomRequest {
                                            player_uuid,
                                            game_version,
                                            protocol_hash,
                                            nonce: challenge.nonce,
                                            solution,
                                        })
                                        .send()
                                        .await;

                                    match create_res {
                                        Ok(resp) => {
                                            if resp.status().is_success() {
                                                if let Ok(data) =
                                                    resp.json::<CreateRoomResponse>().await
                                                {
                                                    let _ = tx_to_bevy
                                                        .send(HubResponse::RoomCreated(data));
                                                }
                                            } else {
                                                let _ = tx_to_bevy.send(HubResponse::Error(
                                                    format!("Status: {}", resp.status()),
                                                ));
                                            }
                                        }
                                        Err(e) => {
                                            let _ =
                                                tx_to_bevy.send(HubResponse::Error(e.to_string()));
                                        }
                                    }
                                } else {
                                    let _ = tx_to_bevy.send(HubResponse::Error(
                                        "Failed to parse challenge response".to_string(),
                                    ));
                                }
                            } else {
                                let _ = tx_to_bevy.send(HubResponse::Error(format!(
                                    "Challenge failed: {}",
                                    resp.status()
                                )));
                            }
                        } else {
                            let _ = tx_to_bevy.send(HubResponse::Error(
                                "Failed to request challenge".to_string(),
                            ));
                        }
                    }
                    HubRequest::JoinRoom { code, player_uuid } => {
                        let res = client
                            .post(format!("{}/v1/rooms/join/{}", worker_hub_url, code))
                            .json(&JoinRoomRequest { player_uuid })
                            .send()
                            .await;
                        match res {
                            Ok(resp) => {
                                if resp.status().is_success() {
                                    if let Ok(data) = resp.json::<JoinRoomResponse>().await {
                                        let _ = tx_to_bevy.send(HubResponse::RoomJoined(data));
                                    }
                                } else {
                                    let _ = tx_to_bevy.send(HubResponse::Error(format!(
                                        "Status: {}",
                                        resp.status()
                                    )));
                                }
                            }
                            Err(e) => {
                                let _ = tx_to_bevy.send(HubResponse::Error(e.to_string()));
                            }
                        }
                    }
                    HubRequest::Ping {
                        installation_id,
                        version,
                        protocol_hash,
                    } => {
                        let res = client
                            .post(format!("{}/v1/ping", worker_hub_url))
                            .json(&PingRequest {
                                installation_id,
                                version,
                                protocol_hash,
                            })
                            .timeout(std::time::Duration::from_secs(3))
                            .send()
                            .await;
                        match res {
                            Ok(resp) if resp.status().is_success() => {
                                if let Ok(data) = resp.json::<PingResponse>().await {
                                    let _ = tx_to_bevy.send(HubResponse::PingResult {
                                        ok: data.ok,
                                        online_players: data.online_players_estimate,
                                        multiplayer_status: data.multiplayer_status,
                                        upgrade_version: data.upgrade_version,
                                    });
                                } else {
                                    let _ = tx_to_bevy.send(HubResponse::PingResult {
                                        ok: false,
                                        online_players: 0,
                                        multiplayer_status: MultiplayerStatus::Unsupported,
                                        upgrade_version: None,
                                    });
                                }
                            }
                            _ => {
                                let _ = tx_to_bevy.send(HubResponse::PingResult {
                                    ok: false,
                                    online_players: 0,
                                    multiplayer_status: MultiplayerStatus::Unsupported,
                                    upgrade_version: None,
                                });
                            }
                        }
                    }
                }
            }
        });
    });

    commands.insert_resource(HubClient {
        tx: tx_to_worker,
        rx: rx_from_worker,
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
        let _ = client.tx.send(HubRequest::Ping {
            installation_id: profile.installation_id,
            version: env!("CARGO_PKG_VERSION").to_string(),
            protocol_hash: protocol_hash.0,
        });
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
