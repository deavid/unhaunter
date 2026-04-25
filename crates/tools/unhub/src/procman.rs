use crate::state::{HubState, ProcManSession};
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{error, info, warn};
use unhub_client::protocol::ProcManMessage;

pub async fn run_procman_listener(state: HubState, addr: SocketAddr) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    info!("ProcMan listener running on {}", addr);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_procman_connection(state, stream, peer_addr).await {
                error!(
                    "Error handling ProcMan connection from {}: {}",
                    peer_addr, e
                );
            }
        });
    }
}

async fn handle_procman_connection(
    state: HubState,
    stream: TcpStream,
    peer_addr: SocketAddr,
) -> anyhow::Result<()> {
    let mut framed = Framed::new(stream, LinesCodec::new_with_max_length(65536));

    // 1. Handshake
    let line = framed
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("Connection closed"))??;
    let hello = serde_json::from_str::<ProcManMessage>(&line)?;

    let (uuid, public_addr, library, idle_pool, rooms, ticket_hmac_secret) = match hello {
        ProcManMessage::ProcManHello {
            uuid,
            public_addr,
            library,
            idle_pool,
            rooms,
            ticket_hmac_secret,
            ..
        } => (
            uuid,
            public_addr,
            library,
            idle_pool,
            rooms,
            ticket_hmac_secret,
        ),
        _ => return Err(anyhow::anyhow!("Expected ProcManHello")),
    };

    // Validate UUID
    {
        let config = state.config.read().await;
        if !config.allowed_procman_uuids.contains(&uuid) {
            warn!("ProcMan {} rejected (not in allow-list)", uuid);
            let msg = ProcManMessage::AuthRejected {
                reason: "UUID not in allow-list".to_string(),
            };
            let _ = framed.send(serde_json::to_string(&msg)?).await;
            return Err(anyhow::anyhow!("ProcMan {} not allowed", uuid));
        }
    }

    info!("ProcMan {} connected from {}", uuid, peer_addr);

    // Accept ProcMan
    let accept = ProcManMessage::ProcManAccepted {
        hub_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    framed.send(serde_json::to_string(&accept)?).await?;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    // Register ProcMan
    let idle_capacity: usize = idle_pool.values().sum();
    state.procmans.insert(
        uuid,
        ProcManSession {
            tx,
            library,
            public_addr,
            idle_capacity,
            last_heartbeat: std::time::Instant::now(),
            ticket_hmac_secret,
        },
    );

    // Register existing rooms
    for room in rooms {
        state.rooms.insert(room.code.clone(), room);
    }

    // Main loop
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                let line = serde_json::to_string(&msg)?;
                framed.send(line).await?;
            }
            result = framed.next() => {
                match result {
                    Some(Ok(line)) => {
                        let msg = serde_json::from_str::<ProcManMessage>(&line)?;
                        handle_message(&state, uuid, msg).await?;
                    }
                    Some(Err(e)) => return Err(e.into()),
                    None => break,
                }
            }
        }
    }

    // Cleanup
    info!("ProcMan {} disconnected", uuid);
    state.procmans.remove(&uuid);

    let mut rooms_to_remove = Vec::new();
    for entry in state.rooms.iter() {
        if entry.value().server_id == uuid {
            rooms_to_remove.push(entry.key().clone());
        }
    }

    for room_code in rooms_to_remove {
        state.rooms.remove(&room_code);
        if let Some((_, ip)) = state.room_to_ip.remove(&room_code)
            && let Some(mut rooms) = state.rooms_by_ip.get_mut(&ip)
        {
            rooms.retain(|c| c != &room_code);
            if rooms.is_empty() {
                drop(rooms);
                state.rooms_by_ip.remove(&ip);
            }
        }
    }

    Ok(())
}

async fn handle_message(
    state: &HubState,
    uuid: uuid::Uuid,
    msg: ProcManMessage,
) -> anyhow::Result<()> {
    if let Some(mut pm) = state.procmans.get_mut(&uuid) {
        pm.last_heartbeat = std::time::Instant::now();
    }
    match msg {
        ProcManMessage::Heartbeat {
            idle_capacity,
            library,
            rooms,
        } => {
            if let Some(mut pm) = state.procmans.get_mut(&uuid) {
                pm.idle_capacity = idle_capacity;
                pm.library = library;
            }
            // Update rooms (could be more efficient)
            for room in rooms {
                state.rooms.insert(room.code.clone(), room);
            }
        }
        ProcManMessage::RoomReady { room } => {
            state.rooms.insert(room.code.clone(), room);
        }
        ProcManMessage::PlayerJoined { room_code, .. } => {
            if let Some(mut room) = state.rooms.get_mut(&room_code) {
                room.player_count += 1;
            }
        }
        ProcManMessage::PlayerLeft { room_code, .. } => {
            if let Some(mut room) = state.rooms.get_mut(&room_code) {
                room.player_count = room.player_count.saturating_sub(1);
            }
        }
        ProcManMessage::RoomStateChanged {
            room_code,
            new_state,
            metadata,
            ..
        } => {
            if let Some(mut room) = state.rooms.get_mut(&room_code) {
                room.state = new_state;
                room.metadata = metadata;
            }
        }
        ProcManMessage::RoomClosed { room_code, .. } => {
            state.rooms.remove(&room_code);
            if let Some((_, ip)) = state.room_to_ip.remove(&room_code)
                && let Some(mut rooms) = state.rooms_by_ip.get_mut(&ip)
            {
                rooms.retain(|c| c != &room_code);
                if rooms.is_empty() {
                    drop(rooms); // Release the lock before removing
                    state.rooms_by_ip.remove(&ip);
                }
            }
        }
        _ => warn!("Unhandled message from ProcMan {}: {:?}", uuid, msg),
    }
    Ok(())
}
