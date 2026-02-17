use crate::state::HubState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use unhub_client::protocol::{
    CreateRoomRequest, CreateRoomResponse, HealthResponse, HubError, JoinRoomRequest,
    JoinRoomResponse, ProcManMessage,
};
use unhub_client::{generate_room_code, generate_room_secret};

pub async fn health(State(state): State<HubState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        hub_version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
    })
}

pub async fn create_room(
    State(state): State<HubState>,
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

    // Select a ProcMan with capacity — extract what we need in a single lookup
    // to avoid a second DashMap get that could race with disconnection.
    let (tx, public_addr) = state
        .procmans
        .iter()
        .find(|pm| pm.game_versions.contains(&payload.game_version) && pm.idle_capacity > 0)
        .map(|pm| (pm.tx.clone(), pm.public_addr.clone()))
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

    // Ask ProcMan to create the room
    tx.send(ProcManMessage::CreateRoom {
        room_code: room_code.clone(),
        secret: secret.clone(),
        game_version: payload.game_version.clone(),
    })
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(HubError {
                error: "internal_error".to_string(),
                message: "Failed to communicate with process manager.".to_string(),
            }),
        )
    })?;

    // WAIT for RoomReady (timeout 5s)
    for _ in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if let Some(room) = state.rooms.get(&room_code) {
            return Ok(Json(CreateRoomResponse {
                code: room_code,
                addr: format!("{}:{}", public_addr, room.port),
                secret: room.secret.clone(),
            }));
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

    Ok(Json(JoinRoomResponse {
        code,
        addr: format!("{}:{}", pm.public_addr, room.port),
        secret: room.secret.clone(),
    }))
}
