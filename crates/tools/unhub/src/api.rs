use crate::state::HubState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use sha2::Digest;
use unhub_client::protocol::{
    CreateRoomRequest, CreateRoomResponse, HealthResponse, HubError, JoinRoomRequest,
    JoinRoomResponse, PingRequest, PingResponse, ProcManMessage,
};
use unhub_client::tickets::{ConnectionTicket, encode_ticket};
use unhub_client::{generate_room_code, generate_room_secret};

pub async fn ping(
    State(state): State<HubState>,
    Json(payload): Json<PingRequest>,
) -> Json<PingResponse> {
    state.active_players.insert(payload.installation_id, ());
    state.active_players.run_pending_tasks();

    Json(PingResponse {
        ok: true,
        online_players_estimate: state.active_players.entry_count() as usize,
    })
}

pub async fn health(State(state): State<HubState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        hub_version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
    })
}

pub async fn challenge(
    State(state): State<HubState>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<unhub_client::protocol::ChallengeRequest>,
) -> Result<Json<unhub_client::protocol::ChallengeResponse>, (StatusCode, Json<HubError>)> {
    let config = state.config.read().await;

    // Check ban list
    if config.banned_uuids.contains(&payload.player_uuid) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(HubError {
                error: "banned".to_string(),
                message: "Banned".to_string(),
            }),
        ));
    }

    // Extract IP (same logic as create_room)
    let client_ip = if config.trust_proxy_headers && addr.ip().is_loopback() {
        headers
            .get("X-Forwarded-For")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<std::net::IpAddr>().ok())
            .unwrap_or(addr.ip())
    } else {
        addr.ip()
    };

    // Enforce max 2 outstanding nonces per UUID and IP
    let mut uuid_count = 0;
    let mut ip_count = 0;
    let mut expired_nonces = Vec::new();
    let now = std::time::Instant::now();

    for entry in state.nonces.iter() {
        if now.duration_since(entry.issued_at).as_secs() > 120 {
            expired_nonces.push(entry.key().clone());
            continue;
        }
        if entry.player_uuid == payload.player_uuid {
            uuid_count += 1;
        }
        if entry.client_ip == client_ip {
            ip_count += 1;
        }
    }

    for nonce in expired_nonces {
        state.nonces.remove(&nonce);
    }

    if uuid_count >= 2 || ip_count >= 2 {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(HubError {
                error: "rate_limited".to_string(),
                message: "Too many challenges".to_string(),
            }),
        ));
    }

    let nonce = uuid::Uuid::new_v4().to_string();
    state.nonces.insert(
        nonce.clone(),
        crate::state::NonceEntry {
            player_uuid: payload.player_uuid,
            client_ip,
            issued_at: std::time::Instant::now(),
        },
    );

    Ok(Json(unhub_client::protocol::ChallengeResponse {
        nonce,
        difficulty: config.pow_difficulty,
    }))
}

pub async fn create_room(
    State(state): State<HubState>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateRoomRequest>,
) -> Result<Json<CreateRoomResponse>, (StatusCode, Json<HubError>)> {
    let config = state.config.read().await;
    if config.banned_uuids.contains(&payload.player_uuid) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(HubError {
                error: "banned".to_string(),
                message: "You are banned from Hub services.".to_string(),
            }),
        ));
    }

    let client_ip = if config.trust_proxy_headers && addr.ip().is_loopback() {
        headers
            .get("X-Forwarded-For")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<std::net::IpAddr>().ok())
            .unwrap_or(addr.ip())
    } else {
        addr.ip()
    };

    if let Some(rooms) = state.rooms_by_ip.get(&client_ip)
        && rooms.len() >= config.max_rooms_per_ip
    {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(HubError {
                error: "room_cap_exceeded".to_string(),
                message: "You have reached the maximum number of active rooms.".to_string(),
            }),
        ));
    }

    let entry = state.nonces.remove(&payload.nonce).map(|(_, v)| v).ok_or((
        StatusCode::BAD_REQUEST,
        Json(HubError {
            error: "invalid_pow".to_string(),
            message: "Invalid or expired nonce".to_string(),
        }),
    ))?;

    if entry.player_uuid != payload.player_uuid || entry.client_ip != client_ip {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(HubError {
                error: "invalid_pow".to_string(),
                message: "Nonce mismatch".to_string(),
            }),
        ));
    }

    if entry.issued_at.elapsed().as_secs() > 120 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(HubError {
                error: "invalid_pow".to_string(),
                message: "Nonce expired".to_string(),
            }),
        ));
    }

    let candidate = format!("{}:{}", payload.nonce, payload.solution);
    let hash = sha2::Sha256::digest(candidate.as_bytes());
    let mut zero_bits = 0;
    for byte in hash {
        if byte == 0 {
            zero_bits += 8;
        } else {
            zero_bits += byte.leading_zeros();
            break;
        }
    }

    if zero_bits < config.pow_difficulty {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(HubError {
                error: "invalid_pow".to_string(),
                message: "Incorrect solution".to_string(),
            }),
        ));
    }

    // Select a ProcMan with capacity — extract what we need in a single lookup
    // to avoid a second DashMap get that could race with disconnection.
    let (tx, public_addr, pm_uuid, ticket_hmac_secret) = state
        .procmans
        .iter()
        .find(|pm| pm.game_versions.contains(&payload.game_version) && pm.idle_capacity > 0)
        .map(|pm| {
            (
                pm.tx.clone(),
                pm.public_addr.clone(),
                *pm.key(),
                pm.ticket_hmac_secret.clone(),
            )
        })
        .ok_or((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HubError {
                error: "no_capacity".to_string(),
                message: "No server capacity available for this game version.".to_string(),
            }),
        ))?;

    // Generate a unique room code, retrying on collision (bounded to avoid
    // infinite loops from bugs in the RNG or an overly full code space).
    let room_code = (0..10)
        .map(|_| generate_room_code())
        .find(|candidate| !state.rooms.contains_key(candidate))
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(HubError {
                error: "code_exhausted".to_string(),
                message: "Failed to generate a unique room code after 10 attempts.".to_string(),
            }),
        ))?;
    let secret = generate_room_secret();

    state
        .rooms_by_ip
        .entry(client_ip)
        .or_default()
        .push(room_code.clone());
    state.room_to_ip.insert(room_code.clone(), client_ip);

    // Ask ProcMan to create the room
    if tx
        .send(ProcManMessage::CreateRoom {
            room_code: room_code.clone(),
            secret: secret.clone(),
            game_version: payload.game_version.clone(),
        })
        .is_err()
    {
        if let Some((_, ip)) = state.room_to_ip.remove(&room_code)
            && let Some(mut rooms) = state.rooms_by_ip.get_mut(&ip)
        {
            rooms.retain(|c| c != &room_code);
            if rooms.is_empty() {
                drop(rooms);
                state.rooms_by_ip.remove(&ip);
            }
        }
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(HubError {
                error: "internal_error".to_string(),
                message: "Failed to communicate with process manager.".to_string(),
            }),
        ));
    }

    // WAIT for RoomReady (timeout 5s)
    for _ in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if let Some(room) = state.rooms.get(&room_code) {
            let exp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 300; // 5 mins

            let ticket_data = ConnectionTicket {
                room_code: room_code.clone(),
                installation_id: pm_uuid,
                player_uuid: payload.player_uuid,
                exp,
            };

            let raw_ticket = encode_ticket(&ticket_data, &ticket_hmac_secret).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(HubError {
                        error: "ticket_error".to_string(),
                        message: format!("Failed to encode connection ticket: {}", e),
                    }),
                )
            })?;

            let ticket = B64.encode(raw_ticket);
            return Ok(Json(CreateRoomResponse {
                code: room_code,
                addr: format!("{}:{}", public_addr, room.port),
                secret: room.secret.clone(),
                ticket,
            }));
        }
    }

    // Timeout cleanup
    if let Some((_, ip)) = state.room_to_ip.remove(&room_code)
        && let Some(mut rooms) = state.rooms_by_ip.get_mut(&ip)
    {
        rooms.retain(|c| c != &room_code);
        if rooms.is_empty() {
            drop(rooms);
            state.rooms_by_ip.remove(&ip);
        }
    }

    Err((
        StatusCode::GATEWAY_TIMEOUT,
        Json(HubError {
            error: "timeout".to_string(),
            message: "Timed out waiting for server allocation.".to_string(),
        }),
    ))
}

pub async fn join_room(
    State(state): State<HubState>,
    Path(code): Path<String>,
    Json(payload): Json<JoinRoomRequest>,
) -> Result<Json<JoinRoomResponse>, (StatusCode, Json<HubError>)> {
    let config = state.config.read().await;
    if config.banned_uuids.contains(&payload.player_uuid) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(HubError {
                error: "banned".to_string(),
                message: "You are banned from Hub services.".to_string(),
            }),
        ));
    }
    drop(config);

    let room = state.rooms.get(&code).ok_or((
        StatusCode::NOT_FOUND,
        Json(HubError {
            error: "code_not_found".to_string(),
            message: format!("Room {} not found.", code),
        }),
    ))?;

    let pm = state.procmans.get(&room.server_id).ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(HubError {
            error: "server_lost".to_string(),
            message: "The server hosting this room is currently disconnected.".to_string(),
        }),
    ))?;

    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 300; // 5 mins

    let ticket_data = ConnectionTicket {
        room_code: code.clone(),
        installation_id: *pm.key(),
        player_uuid: payload.player_uuid,
        exp,
    };

    let raw_ticket = encode_ticket(&ticket_data, &pm.ticket_hmac_secret).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(HubError {
                error: "ticket_error".to_string(),
                message: format!("Failed to encode connection ticket: {}", e),
            }),
        )
    })?;

    let ticket = B64.encode(raw_ticket);

    Ok(Json(JoinRoomResponse {
        code,
        addr: format!("{}:{}", pm.public_addr, room.port),
        secret: room.secret.clone(),
        ticket,
    }))
}
