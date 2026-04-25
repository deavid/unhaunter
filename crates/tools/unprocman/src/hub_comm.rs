use crate::manager::ServerManager;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{error, info, warn};
use unhub_client::protocol::ProcManMessage;

pub async fn run_hub_comm(manager: Arc<ServerManager>) -> anyhow::Result<()> {
    let hub_addr = manager.config.hub_addr.clone();

    loop {
        info!("Connecting to Hub at {}...", hub_addr);
        match TcpStream::connect(&hub_addr).await {
            Ok(stream) => {
                if let Err(e) = handle_hub_connection(stream, manager.clone()).await {
                    error!("Hub connection error: {}", e);
                }
                let mut hub_tx_lock = manager.hub_tx.lock().await;
                *hub_tx_lock = None;
            }
            Err(e) => {
                error!("Failed to connect to Hub: {}. Retrying in 5s...", e);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn handle_hub_connection(
    stream: TcpStream,
    manager: Arc<ServerManager>,
) -> anyhow::Result<()> {
    let mut framed = Framed::new(stream, LinesCodec::new_with_max_length(65536));

    let library = manager.get_library_entries().await;
    let mut rooms_summary = Vec::new();
    {
        let servers = manager.servers.lock().await;
        for s in servers.values() {
            if let (Some(code), Some(secret)) = (&s.room_code, &s.secret) {
                rooms_summary.push(unhub_client::protocol::RoomSummary {
                    code: code.clone(),
                    port: s.port,
                    game_version: unhub_client::GAME_VERSION.to_string(),
                    secret: secret.clone(),
                    state: s.state,
                    player_count: s.player_count,
                    metadata: unhub_client::protocol::RoomMetadata {
                        map: "".into(),
                        difficulty: "".into(),
                    },
                    server_id: manager.config.installation_id,
                });
            }
        }
    }

    let hello = ProcManMessage::ProcManHello {
        uuid: manager.config.installation_id,
        version: env!("CARGO_PKG_VERSION").to_string(),
        library,
        port_range: manager.config.port_range,
        public_addr: manager.config.public_addr.clone(),
        rooms: rooms_summary,
        ticket_hmac_secret: manager.config.ticket_hmac_secret.clone(),
    };

    framed.send(serde_json::to_string(&hello)?).await?;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    {
        let mut hub_tx_lock = manager.hub_tx.lock().await;
        *hub_tx_lock = Some(tx);
    }

    match framed.next().await {
        Some(Ok(line)) => {
            let msg = serde_json::from_str::<ProcManMessage>(&line)?;
            match msg {
                ProcManMessage::ProcManAccepted { hub_version } => {
                    info!("Connected to Hub (version: {})", hub_version);
                }
                ProcManMessage::AuthRejected { reason } => {
                    return Err(anyhow::anyhow!("Auth rejected by Hub: {}", reason));
                }
                _ => return Err(anyhow::anyhow!("Unexpected message during handshake")),
            }
        }
        _ => return Err(anyhow::anyhow!("Connection closed during handshake")),
    }

    let mut heartbeat_interval = tokio::time::interval(std::time::Duration::from_secs(60));
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                let line = serde_json::to_string(&msg)?;
                framed.send(line).await?;
            }
            _ = heartbeat_interval.tick() => {
                let servers = manager.servers.lock().await;
                let library = manager.get_library_entries().await;
                let mut rooms = Vec::new();
                for s in servers.values() {
                    if let (Some(code), Some(secret)) = (&s.room_code, &s.secret) {
                        rooms.push(unhub_client::protocol::RoomSummary {
                            code: code.clone(),
                            port: s.port,
                            game_version: unhub_client::GAME_VERSION.to_string(),
                            secret: secret.clone(),
                            state: s.state,
                            player_count: s.player_count,
                            metadata: unhub_client::protocol::RoomMetadata { map: "".into(), difficulty: "".into() },
                            server_id: manager.config.installation_id,
                        });
                    }
                }
                let hb = ProcManMessage::Heartbeat { library, rooms };
                framed.send(serde_json::to_string(&hb)?).await?;
            }
            result = framed.next() => {
                match result {
                    Some(Ok(line)) => {
                        let msg = serde_json::from_str::<ProcManMessage>(&line)?;
                        handle_hub_message(&manager, &mut framed, msg).await?;
                    }
                    Some(Err(e)) => return Err(e.into()),
                    None => break,
                }
            }
        }
    }

    Ok(())
}

async fn handle_hub_message(
    manager: &Arc<ServerManager>,
    framed: &mut Framed<TcpStream, LinesCodec>,
    msg: ProcManMessage,
) -> anyhow::Result<()> {
    match msg {
        ProcManMessage::CreateRoom {
            room_code,
            secret,
            target_version,
        } => {
            info!(
                "Creating room {} for target version {}",
                room_code, target_version
            );
            let room_code_for_error = room_code.clone();
            match manager.assign_room(room_code, secret, target_version).await {
                Ok(room) => {
                    let resp = ProcManMessage::RoomReady { room };
                    framed.send(serde_json::to_string(&resp)?).await?;
                }
                Err(e) => {
                    error!("Failed to create room: {}", e);
                    let resp = ProcManMessage::CreateRoomFailed {
                        room_code: room_code_for_error,
                        reason: e.to_string(),
                    };
                    framed.send(serde_json::to_string(&resp)?).await?;
                }
            }
        }
        _ => warn!("Unexpected message from Hub: {:?}", msg),
    }
    Ok(())
}
