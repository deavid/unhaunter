use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use unhub_client::protocol::{
    ChallengeRequest, ChallengeResponse, CreateRoomRequest, CreateRoomResponse, JoinRoomRequest,
    JoinRoomResponse,
};
use untypes_core::cli::CliOptions;

#[derive(Resource)]
pub struct HubClient {
    pub tx: Sender<HubRequest>,
    pub rx: Receiver<HubResponse>,
}

pub enum HubRequest {
    CreateRoom {
        player_uuid: uuid::Uuid,
        game_version: String,
    },
    JoinRoom {
        code: String,
        player_uuid: uuid::Uuid,
    },
}

pub enum HubResponse {
    RoomCreated(CreateRoomResponse),
    RoomJoined(JoinRoomResponse),
    Error(String),
}

#[derive(Resource, Default)]
pub struct HubStatus {
    pub last_response: Option<HubResponse>,
    pub is_pending: bool,
}

pub fn setup_hub_client(mut commands: Commands, cli: Res<CliOptions>) {
    let (tx_to_worker, rx_from_bevy) = crossbeam_channel::unbounded::<HubRequest>();
    let (tx_to_bevy, rx_from_worker) = crossbeam_channel::unbounded::<HubResponse>();

    let hub_url = if let Some(url) = &cli.hub_url {
        url.clone()
    } else {
        warn!("Hub URL not configured, using default localhost:3000");
        "http://localhost:3000".to_string()
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
                }
            }
        });
    });

    commands.insert_resource(HubClient {
        tx: tx_to_worker,
        rx: rx_from_worker,
    });
    commands.insert_resource(HubStatus::default());
}

pub fn update_hub_status(mut status: ResMut<HubStatus>, client: Res<HubClient>) {
    while let Ok(resp) = client.rx.try_recv() {
        status.last_response = Some(resp);
        status.is_pending = false;
    }
}
