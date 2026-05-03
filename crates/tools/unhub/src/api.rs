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
    JoinRoomResponse, MultiplayerStatus, PingRequest, PingResponse, ProcManMessage,
};
use unhub_client::tickets::{ConnectionTicket, encode_ticket};
use unhub_client::{generate_room_code, generate_room_secret};

/// Format a raw IP string and port into the `IP:port` / `[IPv6]:port` form.
fn format_addr_with_port(ip: &str, port: u16) -> String {
    if ip.contains(':') {
        format!("[{}]:{}", ip, port)
    } else {
        format!("{}:{}", ip, port)
    }
}

fn infer_channel(version: &str) -> &'static str {
    if version.contains("-beta") {
        "beta"
    } else if version.ends_with("-alpha") {
        "alpha"
    } else if version.ends_with("-dev") {
        "dev"
    } else {
        "stable"
    }
}

fn is_stable_version(version: &str) -> bool {
    !version.contains('-')
}

fn entry_matches_client_pool(
    entry_version: &str,
    client_channel: &str,
    client_version: &str,
    client_hash: &str,
    entry_hash: &str,
) -> bool {
    match client_channel {
        "dev" | "alpha" => entry_version == client_version && entry_hash == client_hash,
        "beta" => {
            entry_hash == client_hash && matches!(infer_channel(entry_version), "beta" | "stable")
        }
        _ => entry_hash == client_hash && infer_channel(entry_version) == "stable",
    }
}

fn parse_semver(version: &str) -> Option<semver::Version> {
    let normalized = version.trim().trim_start_matches('v');
    match semver::Version::parse(normalized) {
        Ok(v) => Some(v),
        Err(e) => {
            tracing::warn!("Failed to parse semver '{}': {}", version, e);
            None
        }
    }
}

fn is_valid_version_string(version: &str) -> bool {
    if version.is_empty() || version.len() > 128 {
        return false;
    }

    version
        .chars()
        .all(|c| matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '/' | '_' | ':' | '.'))
}

fn evaluate_client_status(
    state: &HubState,
    client_version: &str,
    client_hash: &str,
) -> (MultiplayerStatus, Option<String>) {
    let client_channel = infer_channel(client_version);
    let client_semver = parse_semver(client_version);

    let mut has_server = false;
    let mut best_pool_match: Option<(String, semver::Version)> = None;
    let mut best_stable: Option<(String, semver::Version)> = None;

    for pm in state.procmans.iter() {
        for entry in &pm.library {
            if entry_matches_client_pool(
                &entry.version,
                client_channel,
                client_version,
                client_hash,
                &entry.protocol_hash,
            ) {
                has_server = true;
                if let Some(v) = parse_semver(&entry.version) {
                    let replace = best_pool_match
                        .as_ref()
                        .map(|(_, current)| v > *current)
                        .unwrap_or(true);
                    if replace {
                        best_pool_match = Some((entry.version.clone(), v));
                    }
                }
            }

            if is_stable_version(&entry.version)
                && let Some(v) = parse_semver(&entry.version)
            {
                let replace = best_stable
                    .as_ref()
                    .map(|(_, current)| v > *current)
                    .unwrap_or(true);
                if replace {
                    best_stable = Some((entry.version.clone(), v));
                }
            }
        }
    }

    if let Some(client_semver) = client_semver {
        // 1. Newer version in the same compatible pool (same hash group).
        if let Some((best_pool_version, best_pool_semver)) = &best_pool_match
            && *best_pool_semver > client_semver
        {
            return (
                MultiplayerStatus::UpdateAvailable,
                Some(best_pool_version.clone()),
            );
        }

        // 2. Newer stable version overall: recommend upgrade if old hash is
        // still served; otherwise caller will get Unsupported below.
        if let Some((best_stable_version, best_stable_semver)) = &best_stable
            && *best_stable_semver > client_semver
            && has_server
        {
            return (
                MultiplayerStatus::UpdateRecommended,
                Some(best_stable_version.clone()),
            );
        }

        // 3. Stable and still served with no newer stable available.
        if client_channel == "stable" && has_server {
            return (MultiplayerStatus::UpToDate, None);
        }
    } else {
        tracing::warn!(
            "Client version '{}' is not semver-parseable; using degraded ping status logic",
            client_version
        );
    }

    if has_server {
        (MultiplayerStatus::UpToDate, None)
    } else {
        (MultiplayerStatus::Unsupported, None)
    }
}

pub async fn ping(
    State(state): State<HubState>,
    Json(payload): Json<PingRequest>,
) -> Json<PingResponse> {
    if payload.installation_id.is_nil() || !is_valid_version_string(&payload.version) {
        return Json(PingResponse {
            ok: false,
            online_players_estimate: state.active_players.entry_count() as usize,
            multiplayer_status: MultiplayerStatus::Unsupported,
            upgrade_version: None,
        });
    }

    if let Some(existing_session) = state.active_sessions.get(&payload.installation_id)
        && existing_session != payload.session_id
        && state.player_to_room.contains_key(&payload.installation_id)
    {
        return Json(PingResponse {
            ok: true,
            online_players_estimate: state.active_players.entry_count() as usize,
            multiplayer_status: MultiplayerStatus::Conflict,
            upgrade_version: None,
        });
    }

    state.active_players.insert(payload.installation_id, ());
    state
        .active_sessions
        .insert(payload.installation_id, payload.session_id);
    state
        .players_1h
        .insert(payload.installation_id, payload.version.clone());
    state
        .players_24h
        .insert(payload.installation_id, payload.version.clone());
    state.active_players.run_pending_tasks();
    state.players_1h.run_pending_tasks();
    state.players_24h.run_pending_tasks();

    let (multiplayer_status, upgrade_version) =
        evaluate_client_status(&state, &payload.version, &payload.protocol_hash);

    Json(PingResponse {
        ok: true,
        online_players_estimate: state.active_players.entry_count() as usize,
        multiplayer_status,
        upgrade_version,
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
        if now.duration_since(entry.issued_at).as_secs() > 10 {
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
    if state.player_to_room.contains_key(&payload.player_uuid) {
        return Err((
            StatusCode::CONFLICT,
            Json(HubError {
                error: "already_in_room".to_string(),
                message: "You are already in an active room.".to_string(),
            }),
        ));
    }
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

    if entry.issued_at.elapsed().as_secs() > 10 {
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

    // Select ProcMan by protocol hash, preferring the candidate that reports
    // the highest semver among versions allowed by channel-boundary rules.
    let client_channel = infer_channel(&payload.game_version);
    let mut selected: Option<(
        tokio::sync::mpsc::UnboundedSender<ProcManMessage>,
        Vec<String>,
        uuid::Uuid,
        String,
        String,
        Option<semver::Version>,
    )> = None;

    for pm in state.procmans.iter() {
        let mut best_target_version: Option<String> = None;
        let mut pm_best_semver: Option<semver::Version> = None;

        for entry in &pm.library {
            if !entry_matches_client_pool(
                &entry.version,
                client_channel,
                &payload.game_version,
                &payload.protocol_hash,
                &entry.protocol_hash,
            ) {
                continue;
            }

            if let Some(v) = parse_semver(&entry.version) {
                let replace = pm_best_semver
                    .as_ref()
                    .map(|current| v > *current)
                    .unwrap_or(true);
                if replace {
                    pm_best_semver = Some(v);
                    best_target_version = Some(entry.version.clone());
                }
            } else if best_target_version.is_none() {
                best_target_version = Some(entry.version.clone());
            }
        }

        let Some(target_version) = best_target_version else {
            continue;
        };

        let candidate = (
            pm.tx.clone(),
            pm.public_addrs.clone(),
            *pm.key(),
            pm.ticket_hmac_secret.clone(),
            target_version,
            pm_best_semver,
        );

        let should_replace = match &selected {
            None => true,
            Some((_, _, _, _, _, selected_semver)) => match (&candidate.5, selected_semver) {
                (Some(candidate_v), Some(selected_v)) => candidate_v > selected_v,
                (Some(_), None) => true,
                _ => false,
            },
        };

        if should_replace {
            selected = Some(candidate);
        }
    }

    let (tx, public_addrs, pm_uuid, ticket_hmac_secret, target_version, _) = selected.ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(HubError {
            error: "no_capacity".to_string(),
            message: format!(
                "No compatible server available for client version {} (hash {}).",
                payload.game_version, payload.protocol_hash
            ),
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
            target_version,
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
            let addrs = public_addrs
                .iter()
                .map(|ip| format_addr_with_port(ip, room.port))
                .collect();
            return Ok(Json(CreateRoomResponse {
                code: room_code,
                addrs,
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
    if state.player_to_room.contains_key(&payload.player_uuid) {
        return Err((
            StatusCode::CONFLICT,
            Json(HubError {
                error: "already_in_room".to_string(),
                message: "You are already in an active room.".to_string(),
            }),
        ));
    }
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

    // Verify protocol hash
    let mut version_found = false;
    for entry in &pm.library {
        if entry.version == room.game_version {
            version_found = true;
            if entry.protocol_hash != payload.protocol_hash {
                return Err((
                    StatusCode::CONFLICT,
                    Json(HubError {
                        error: "protocol_mismatch".to_string(),
                        message: format!(
                            "Protocol hash mismatch for room {}. Client has {}, but room requires {}.",
                            code, payload.protocol_hash, entry.protocol_hash
                        ),
                    }),
                ));
            }
            break;
        }
    }

    if !version_found {
        tracing::error!(
            "Room {} uses version {}, but it's not in the hosting ProcMan's library",
            code,
            room.game_version
        );
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HubError {
                error: "room_version_unavailable".to_string(),
                message: format!(
                    "The room requires game version {}, but the hosting server cannot currently verify its protocol. Please try again later.",
                    room.game_version
                ),
            }),
        ));
    }

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

    let addrs = pm
        .public_addrs
        .iter()
        .map(|ip| format_addr_with_port(ip, room.port))
        .collect();
    Ok(Json(JoinRoomResponse {
        code,
        addrs,
        secret: room.secret.clone(),
        ticket,
    }))
}
