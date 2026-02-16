use bevy::prelude::*;
use unhub_client::protocol::{CreateRoomRequest, CreateRoomResponse, JoinRoomRequest, JoinRoomResponse};
use crossbeam_channel::{Receiver, Sender};
use untypes_core::cli::CliOptions;
use bevy_persistent::Persistent;
use unprofile_core::profile::PlayerProfileData;
use unhub_client::generate_codename;
use rand::Rng;

#[derive(Resource)]
pub struct HubClient {
    pub tx: Sender<HubRequest>,
    pub rx: Receiver<HubResponse>,
}

pub enum HubRequest {
    CreateRoom { player_uuid: uuid::Uuid, game_version: String },
    JoinRoom { code: String, player_uuid: uuid::Uuid },
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
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let client = reqwest::Client::new();
            while let Ok(req) = rx_from_bevy.recv() {
                match req {
                    HubRequest::CreateRoom { player_uuid, game_version } => {
                        let res = client.post(format!("{}/v1/rooms/create", worker_hub_url))
                            .json(&CreateRoomRequest { player_uuid, game_version })
                            .send()
                            .await;
                        match res {
                            Ok(resp) => {
                                if resp.status().is_success() {
                                    if let Ok(data) = resp.json::<CreateRoomResponse>().await {
                                        let _ = tx_to_bevy.send(HubResponse::RoomCreated(data));
                                    }
                                } else {
                                    let _ = tx_to_bevy.send(HubResponse::Error(format!("Status: {}", resp.status())));
                                }
                            }
                            Err(e) => {
                                let _ = tx_to_bevy.send(HubResponse::Error(e.to_string()));
                            }
                        }
                    }
                    HubRequest::JoinRoom { code, player_uuid } => {
                        let res = client.post(format!("{}/v1/rooms/join/{}", worker_hub_url, code))
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
                                    let _ = tx_to_bevy.send(HubResponse::Error(format!("Status: {}", resp.status())));
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

pub fn initialize_nickname(mut profile: ResMut<Persistent<PlayerProfileData>>) {
    if profile.nickname_letter.is_none() {
        let mut rng = rand::rng();
        let letter = (b'A' + (rng.next_u32() % 26) as u8) as char;
        profile.nickname_letter = Some(letter);
        profile.nickname_attempt = 0;
        let _ = profile.persist();
        info!("Initialized nickname to {}", generate_codename(letter, 0));
    }
}
