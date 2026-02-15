use crate::resources::{ClientConnection, HandshakeState, NetworkConn, PlayerRegistry};
use bevy::prelude::*;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unmetrics_core::metrics::SendMetric;
use unnet_core::messages::{NetworkDataEvent, NetworkMessage, SendNetworkMessage};
use unnet_core::network_id::NetworkId;
use unnet_core::resources::{HostGone, LobbyData, LocalPlayer};
use unplayer_core::components::PlayerInactive;
use untags_core::tags::PlayerTag;
use untypes_core::cli::{CliOptions, NetMode};
use untypes_core::difficulty::Difficulty;
use untypes_core::states::{AppState, GameState};

use crate::metrics;
use unghost_core::resources::ghost_guess::GhostGuess;
use unnet_core::messages::PlayerJoinedEvent;
use unnet_core::resources::LobbyPlayer;
use unplayer_core::components::{Hiding, PlayerDisconnected, PlayerSpectating};
use unsummary_core::summary::SummaryData;
use untruck_core::types::repellent_tracker::RepellentCraftTracker;

pub(crate) fn session_roster_system(
    mut lobby_data: ResMut<LobbyData>,
    mut ev_player_joined: MessageReader<PlayerJoinedEvent>,
    cli: Res<CliOptions>,
    local_player: Res<LocalPlayer>,
    mut commands: Commands,
    room_owner: Option<Res<unnet_core::resources::RoomOwner>>,
) {
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        return;
    }

    let mut changed = false;

    // Update player list based on PlayerJoinedEvent
    let mut players_to_add = vec![];
    for ev in ev_player_joined.read() {
        if let Some(player) = lobby_data.players.iter_mut().find(|p| p.id == ev.id) {
            if !player.connected {
                player.connected = true;
                changed = true;
            }
        } else {
            players_to_add.push(ev.id);
        }
    }
    for id in players_to_add {
        let next_tint = (lobby_data.players.len() % 9) as u8;
        lobby_data.players.push(LobbyPlayer {
            id,
            tint_color_index: next_tint,
            connected: true,
        });
        changed = true;

        // Dedicated server: Assign first client as RoomOwner if none exists
        if cli.is_headless() && room_owner.is_none() {
            commands.insert_resource(unnet_core::resources::RoomOwner(id));
            info!("Network: First client {:?} assigned as RoomOwner", id);
        }
    }

    // Ensure host is always in the list
    if let Some(host_id) = local_player.0
        && !lobby_data.players.iter().any(|p| p.id == host_id)
    {
        lobby_data.players.insert(
            0,
            LobbyPlayer {
                id: host_id,
                tint_color_index: 0,
                connected: true,
            },
        );
        changed = true;
    }

    if !changed {
        lobby_data.bypass_change_detection();
    }
}

pub(crate) fn lobby_broadcast_state_system(
    lobby_data: Res<LobbyData>,
    mut ev_send: MessageWriter<SendNetworkMessage>,
    cli: Res<CliOptions>,
    time: Res<Time>,
    mut last_broadcast: Local<f32>,
    room_owner: Option<Res<unnet_core::resources::RoomOwner>>,
) {
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        return;
    }

    // Periodically broadcast state (every 500ms)
    if time.elapsed_secs() - *last_broadcast > 0.5 {
        ev_send.write(SendNetworkMessage(NetworkMessage::LobbyState {
            players: lobby_data.players.clone(),
            selected_map: lobby_data.selected_map.clone(),
            selected_difficulty: lobby_data.selected_difficulty.clone(),
            room_owner: room_owner.map(|r| r.0),
        }));
        *last_broadcast = time.elapsed_secs();
    }
}

pub(crate) fn client_heartbeat_system(
    mut conn: ResMut<NetworkConn>,
    current_app_state: Res<State<AppState>>,
    cli: Res<CliOptions>,
) {
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        return;
    }
    // Only send if connected (Client variant)
    if let NetworkConn::Client {
        handshake: HandshakeState::Completed,
        write_queue,
        ..
    } = &mut *conn
    {
        write_queue.push_back(NetworkMessage::Heartbeat {
            app_state: *current_app_state.get(),
        });
    }
}

pub(crate) fn host_liveness_system(
    conn: Res<NetworkConn>,
    mut commands: Commands,
    time: Res<Time>,
    query_inactive: Query<Entity, With<PlayerInactive>>,
    query_players: Query<(Entity, &NetworkId), With<PlayerTag>>,
) {
    if let NetworkConn::Host { clients, .. } = &*conn {
        let now = time.elapsed_secs();

        for client in clients {
            if let Some(player_id) = client.associated_id {
                let heartbeat_timeout = (now - client.last_heartbeat) > 5.0;
                let input_timeout = (now - client.last_input) > 180.0;
                let left_mission = client
                    .client_app_state
                    .is_some_and(|s| s != AppState::InGame);
                let inactive = heartbeat_timeout || input_timeout || left_mission;

                // Find entity
                let entity = query_players
                    .iter()
                    .find(|(_, id)| *id == &player_id)
                    .map(|(e, _)| e);

                if let Some(entity) = entity {
                    let is_inactive_comp = query_inactive.contains(entity);
                    if inactive && !is_inactive_comp {
                        commands.entity(entity).insert(PlayerInactive);
                        info!(
                            "Player {:?} marked INACTIVE (HB: {:.1}s, Input: {:.1}s ago)",
                            player_id,
                            now - client.last_heartbeat,
                            now - client.last_input
                        );
                    } else if !inactive && is_inactive_comp {
                        commands.entity(entity).remove::<PlayerInactive>();
                        info!("Player {:?} marked ACTIVE", player_id);
                    }
                }
            }
        }
    }
}

pub(crate) fn host_process_heartbeats_system(
    mut conn: ResMut<NetworkConn>,
    mut ev_reader: MessageReader<NetworkDataEvent>,
    time: Res<Time>,
) {
    let now = time.elapsed_secs();
    for ev in ev_reader.read() {
        if let Some(source_id) = ev.source {
            // Update the client connection
            if let NetworkConn::Host { clients, .. } = &mut *conn
                && let Some(client) = clients
                    .iter_mut()
                    .find(|c| c.associated_id == Some(source_id))
            {
                client.last_heartbeat = now;
                match &ev.message {
                    NetworkMessage::Heartbeat { app_state } => {
                        client.client_app_state = Some(*app_state);
                    }
                    NetworkMessage::PlayerInput {
                        movement,
                        run,
                        interact,
                        use_right_hand,
                        use_left_hand,
                        target_position,
                        aim_direction,
                        ..
                    } => {
                        let is_moving = movement[0].abs() > 0.001 || movement[1].abs() > 0.001;
                        let aim_diff = (aim_direction[0] - client.last_aim_direction[0]).abs()
                            + (aim_direction[1] - client.last_aim_direction[1]).abs();
                        let is_aiming = aim_diff > 0.001;
                        let is_active = is_moving
                            || is_aiming
                            || *run
                            || *interact
                            || *use_right_hand
                            || *use_left_hand
                            || target_position.is_some();
                        if is_active {
                            client.last_input = now;
                        }
                        client.last_aim_direction = *aim_direction;
                    }
                    _ => {
                        // Any other message from client to host counts as activity
                        client.last_input = now;
                    }
                }
            }
        }
    }
}

pub(crate) fn host_status_updater_system(
    mut ev_send: MessageWriter<SendNetworkMessage>,
    app_state: Res<State<AppState>>,
    conn: Res<NetworkConn>,
    cli: Res<CliOptions>,
    summary_data: Res<SummaryData>,
    ghost_guess: Res<GhostGuess>,
    repellent_tracker: Res<RepellentCraftTracker>,
    query_spectating: Query<Entity, With<PlayerSpectating>>,
    query_players: Query<(Entity, &NetworkId), With<PlayerTag>>,
    mut lobby_data: ResMut<LobbyData>,
) {
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        return;
    }

    // Update local lobby data
    lobby_data.host_app_state = Some(*app_state.get());
    lobby_data.mission_elapsed_secs = summary_data.time_taken_secs;
    lobby_data.evidences_found_count = ghost_guess.evidences_found.len() as u32;
    lobby_data.repellent_used = repellent_tracker.crafted_count;

    // Build player statuses
    let mut player_statuses = Vec::new();
    if let NetworkConn::Host { clients, .. } = &*conn {
        for client in clients {
            if let Some(id) = client.associated_id {
                let is_alive = query_players
                    .iter()
                    .find(|(_, net_id)| **net_id == id)
                    .map(|(e, _)| !query_spectating.contains(e))
                    .unwrap_or(true);

                player_statuses.push(unnet_core::messages::PlayerStatusInfo {
                    id,
                    is_alive,
                    is_in_lobby: client
                        .client_app_state
                        .map(|s| s == AppState::Lobby || s == AppState::MainMenu)
                        .unwrap_or(false),
                });
            }
        }
    }

    // Include the host (local player) in player_statuses as well
    // The host is always id 1
    let host_is_alive = query_players
        .iter()
        .find(|(_, net_id)| **net_id == NetworkId(1))
        .map(|(e, _)| !query_spectating.contains(e))
        .unwrap_or(true);

    player_statuses.push(unnet_core::messages::PlayerStatusInfo {
        id: NetworkId(1),
        is_alive: host_is_alive,
        is_in_lobby: *app_state.get() == AppState::Lobby || *app_state.get() == AppState::MainMenu,
    });

    // Also update host's local lobby_data with the player statuses so they show up in UI
    lobby_data.player_statuses = player_statuses.clone();

    ev_send.write(SendNetworkMessage(NetworkMessage::HostStatus {
        app_state: *app_state.get(),
        match_time_elapsed: summary_data.time_taken_secs,
        evidences_found: ghost_guess.evidences_found.len() as u32,
        repellent_used: repellent_tracker.crafted_count,
        player_statuses,
    }));
}

pub(crate) fn client_state_bootstrap_system(
    mut ev_reader: MessageReader<NetworkDataEvent>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut lobby_data: ResMut<LobbyData>,
    current_state: Res<State<AppState>>,
    cli: Res<CliOptions>,
) {
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        return;
    }

    for ev in ev_reader.read() {
        match &ev.message {
            NetworkMessage::HostStatus {
                app_state,
                match_time_elapsed,
                evidences_found,
                repellent_used,
                player_statuses,
            } => {
                lobby_data.host_app_state = Some(*app_state);
                lobby_data.mission_elapsed_secs = *match_time_elapsed;
                lobby_data.evidences_found_count = *evidences_found;
                lobby_data.repellent_used = *repellent_used;
                lobby_data.player_statuses = player_statuses.clone();
            }
            NetworkMessage::Snapshot(msg) => {
                // If host went to Lobby, follow
                if msg.app_state == AppState::Lobby && *current_state.get() != AppState::Lobby {
                    next_app_state.set(AppState::Lobby);
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn startup_network_system(
    cli: Res<CliOptions>,
    mut conn: ResMut<NetworkConn>,
    mut local_id: ResMut<LocalPlayer>,
    mut commands: Commands,
    mut player_registry: ResMut<PlayerRegistry>,
) {
    let measure = metrics::STARTUP_NETWORK_SYSTEM.time_measure();
    if cli.is_headless() {
        player_registry.next_id = 1;
    } else {
        player_registry.next_id = 2;
    }
    match &cli.net_mode {
        NetMode::Offline => {
            *conn = NetworkConn::Disconnected;
            local_id.0 = Some(NetworkId(1));
            commands.insert_resource(unnet_core::resources::RoomOwner(NetworkId(1)));
        }
        NetMode::Host {
            port,
            bind_addresses,
        } => {
            if cli.is_headless() {
                local_id.0 = None;
            } else {
                local_id.0 = Some(NetworkId(1));
                commands.insert_resource(unnet_core::resources::RoomOwner(NetworkId(1)));
            }
            let mut listeners = Vec::new();
            for addr_str in bind_addresses {
                let addrs = format!("{}:{}", addr_str, port);
                match TcpListener::bind(&addrs) {
                    Ok(listener) => {
                        let addr = listener
                            .local_addr()
                            .map(|a| a.to_string())
                            .unwrap_or_else(|_| "unknown".to_string());
                        if let Err(e) = listener.set_nonblocking(true) {
                            error!("Failed to set listener non-blocking: {}", e);
                        } else {
                            info!("Network: Listening on {}", addr);
                            listeners.push(listener);
                        }
                    }
                    Err(e) => {
                        error!("Network: Failed to bind to {}: {}", addrs, e);
                    }
                }
            }
            if listeners.is_empty() {
                error!("Network: Failed to bind to any address on port {}", port);
                *conn = NetworkConn::Disconnected;
            } else {
                *conn = NetworkConn::Host {
                    listeners,
                    clients: Vec::new(),
                };
            }
        }
        NetMode::Join { address } => {
            info!("Network: Connecting to {}...", address);
            match TcpStream::connect(address) {
                Ok(stream) => {
                    if let Err(e) = stream.set_nonblocking(true) {
                        error!("Failed to set stream non-blocking: {}", e);
                    } else {
                        if let Err(e) = stream.set_nodelay(true) {
                            error!("Failed to set TCP_NODELAY: {}", e);
                        }
                        info!("Network: Connected to {}", address);
                        *conn = NetworkConn::Client {
                            stream,
                            read_buffer: String::new(),
                            write_queue: VecDeque::new(),
                            handshake: HandshakeState::None,
                        };
                    }
                }
                Err(e) => error!("Network: Failed to connect to {}: {}", address, e),
            }
        }
    }
    measure.end_ms();
}

pub(crate) fn network_io_system(
    mut conn: ResMut<NetworkConn>,
    mut ev_writer: MessageWriter<NetworkDataEvent>,
    mut ev_disconnect: MessageWriter<unnet_core::messages::NetworkDisconnectEvent>,
    mut ev_send: MessageReader<unnet_core::messages::SendNetworkMessage>,
    mut player_registry: ResMut<crate::resources::PlayerRegistry>,
    time: Res<Time>,
    mut recv_buffer: Local<Vec<u8>>,
    mut last_connection_check: Local<f32>,
) {
    if recv_buffer.len() != 65536 {
        *recv_buffer = vec![0u8; 65536];
    }
    let now = time.elapsed_secs();
    let measure = metrics::NETWORK_IO.time_measure();
    let send_msgs: Vec<NetworkMessage> = ev_send.read().map(|m| m.0.clone()).collect();

    // 1a. Enqueue event-based messages to ALL clients
    for msg in &send_msgs {
        conn.host_broadcast(msg.clone());
    }

    match &mut *conn {
        NetworkConn::Disconnected => {}
        NetworkConn::Host { listeners, clients } => {
            // 2. Accept new connections from listeners
            // Throttling acceptance (every 200ms) to avoid spamming syscalls on high-FPS dedicated servers
            if now - *last_connection_check > 0.2 {
                *last_connection_check = now;
                for listener in listeners.iter() {
                    loop {
                        match listener.accept() {
                            Ok((stream, addr)) => {
                                info!("Network: Client connected from {}", addr);
                                if let Err(e) = stream.set_nonblocking(true) {
                                    error!("Failed to set client stream non-blocking: {}", e);
                                    continue;
                                }
                                if let Err(e) = stream.set_nodelay(true) {
                                    error!("Failed to set TCP_NODELAY for client: {}", e);
                                }
                                clients.push(crate::resources::ClientConnection {
                                    stream,
                                    read_buffer: String::new(),
                                    write_queue: VecDeque::new(),
                                    handshake: HandshakeState::None,
                                    installation_id: None,
                                    associated_id: None,
                                    needs_full_sync: false,
                                    last_heartbeat: now,
                                    last_input: now,
                                    last_aim_direction: [0.0, 0.0],
                                    client_app_state: None,
                                });
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                            Err(e) => {
                                error!("Network: Accept error: {}", e);
                                break;
                            }
                        }
                    }
                }
            }

            // 3. Read/write each client
            let mut to_remove = Vec::new();
            for (idx, client) in clients.iter_mut().enumerate() {
                let closed = do_client_io(
                    client,
                    &mut ev_writer,
                    &mut player_registry,
                    &mut recv_buffer,
                );
                if closed {
                    to_remove.push(idx);
                }
            }

            // Phase 2: Handle duplicate connections (same NetworkId)
            let mut seen_ids: HashMap<NetworkId, usize> = HashMap::new();
            let mut duplicates_to_remove = Vec::new();
            for (idx, client) in clients.iter().enumerate() {
                if to_remove.contains(&idx) {
                    continue;
                }
                if let Some(id) = client.associated_id {
                    match seen_ids.entry(id) {
                        Entry::Occupied(_) => {
                            // Kick the newcomer (current idx), keep the old one
                            duplicates_to_remove.push(idx);
                            error!(
                                "Network: Duplicate connection attempt for {:?}. Kicking newcomer.",
                                id
                            );
                        }
                        Entry::Vacant(v) => {
                            v.insert(idx);
                        }
                    }
                }
            }

            // Combine and remove
            let mut all_removals: Vec<(usize, bool)> =
                to_remove.iter().map(|&i| (i, true)).collect();
            all_removals.extend(duplicates_to_remove.iter().map(|&i| (i, false)));
            all_removals.sort_by_key(|k| k.0);
            all_removals.dedup_by_key(|k| k.0);

            for (idx, emit_disconnect) in all_removals.into_iter().rev() {
                let removed = clients.remove(idx);
                if emit_disconnect && let Some(id) = removed.associated_id {
                    ev_disconnect.write(unnet_core::messages::NetworkDisconnectEvent { id });
                }
                info!("Network: Client {:?} disconnected", removed.associated_id);
            }
        }
        NetworkConn::Client {
            stream,
            read_buffer,
            write_queue,
            handshake,
        } => {
            // 1a. Enqueue event-based messages
            for msg in &send_msgs {
                write_queue.push_back(msg.clone());
            }

            // 2. Read/write the single connection
            let mut closed = false;
            // --- Read ---
            loop {
                match (stream).read(&mut recv_buffer) {
                    Ok(0) => {
                        info!("Network: Connection closed by host");
                        closed = true;
                        break;
                    }
                    Ok(n) => {
                        if let Ok(s) = std::str::from_utf8(&recv_buffer[..n]) {
                            read_buffer.push_str(s);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        break;
                    }
                    Err(e) => {
                        error!("Network: Read error: {}", e);
                        closed = true;
                        break;
                    }
                }
            }

            while let Some(pos) = read_buffer.find('\n') {
                {
                    let line = read_buffer[..pos].trim();
                    if !line.is_empty() {
                        match serde_json::from_str::<NetworkMessage>(line) {
                            Ok(message) => {
                                ev_writer.write(NetworkDataEvent {
                                    message,
                                    source: None,
                                });
                            }
                            Err(e) => {
                                error!(
                                    "Network: Failed to parse JSON message: {}. Line: {}",
                                    e, line
                                );
                            }
                        }
                    }
                }
                read_buffer.replace_range(..pos + 1, "");
            }

            if !closed {
                // --- Write ---
                while let Some(msg) = write_queue.pop_front() {
                    match serde_json::to_string(&msg) {
                        Ok(mut json) => {
                            json.push('\n');
                            if let Err(e) = (stream).write_all(json.as_bytes()) {
                                error!("Network: Write error: {}", e);
                                closed = true;
                                break;
                            }
                        }
                        Err(e) => error!("Network: Serialization error: {}", e),
                    }
                }
            }

            if closed {
                if *handshake == HandshakeState::HelloSent {
                    error!(
                        "Network: Connection closed by host during handshake. You may already be connected from another instance."
                    );
                }
                *conn = NetworkConn::Disconnected;
            }
        }
    }
    measure.end_ms();
}

fn do_client_io(
    client: &mut ClientConnection,
    ev_writer: &mut MessageWriter<NetworkDataEvent>,
    player_registry: &mut PlayerRegistry,
    recv_buffer: &mut [u8],
) -> bool {
    let mut closed = false;
    // --- Read ---
    loop {
        match (&client.stream).read(recv_buffer) {
            Ok(0) => {
                info!("Network: Connection closed by peer");
                closed = true;
                break;
            }
            Ok(n) => {
                if let Ok(s) = std::str::from_utf8(&recv_buffer[..n]) {
                    client.read_buffer.push_str(s);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                break;
            }
            Err(e) => {
                error!("Network: Read error: {}", e);
                closed = true;
                break;
            }
        }
    }

    while let Some(pos) = client.read_buffer.find('\n') {
        {
            let line = client.read_buffer[..pos].trim();
            if !line.is_empty() {
                match serde_json::from_str::<NetworkMessage>(line) {
                    Ok(message) => match &message {
                        NetworkMessage::Hello {
                            installation_id, ..
                        } if client.handshake == HandshakeState::None => {
                            let id = player_registry.get_or_assign(*installation_id);
                            client.associated_id = Some(id);
                            client.installation_id = Some(*installation_id);
                            ev_writer.write(NetworkDataEvent {
                                message,
                                source: Some(id),
                            });
                        }
                        _ if client.handshake == HandshakeState::Completed => {
                            ev_writer.write(NetworkDataEvent {
                                message,
                                source: client.associated_id,
                            });
                        }
                        _ => {
                            warn!("Dropping pre-handshake message from unidentified client");
                        }
                    },
                    Err(e) => {
                        error!(
                            "Network: Failed to parse JSON message: {}. Line: {}",
                            e, line
                        );
                    }
                }
            }
        }
        client.read_buffer.replace_range(..pos + 1, "");
    }

    if !closed {
        // --- Write ---
        while let Some(msg) = client.write_queue.pop_front() {
            match serde_json::to_string(&msg) {
                Ok(mut json) => {
                    json.push('\n');
                    if let Err(e) = (&client.stream).write_all(json.as_bytes()) {
                        error!("Network: Write error: {}", e);
                        closed = true;
                        break;
                    }
                }
                Err(e) => error!("Network: Serialization error: {}", e),
            }
        }
    }
    closed
}

pub(crate) fn handshake_handler_system(
    mut conn: ResMut<NetworkConn>,
    mut ev_reader: MessageReader<NetworkDataEvent>,
    mut ev_player_joined: MessageWriter<unnet_core::messages::PlayerJoinedEvent>,
    cli: Res<CliOptions>,
    mut local_id: ResMut<LocalPlayer>,
    mut pending_map: ResMut<crate::resources::PendingMapLoad>,
    mut current_difficulty: ResMut<CurrentDifficulty>,
    mut commands: Commands,
    query_disconnected: Query<
        (Entity, &NetworkId),
        With<unplayer_core::components::PlayerDisconnected>,
    >,
    runtime_installation_id: Option<Res<unprofile_core::profile::RuntimeInstallationId>>,
    current_app_state: Res<State<AppState>>,
    summary_data: Res<SummaryData>,
    ghost_guess: Res<GhostGuess>,
    repellent_tracker: Res<RepellentCraftTracker>,
) {
    let measure = metrics::HANDSHAKE_HANDLER.time_measure();
    let mut welcomes_to_send = Vec::new();

    match &mut *conn {
        NetworkConn::Disconnected => {}
        NetworkConn::Host { clients, .. } => {
            for ev in ev_reader.read() {
                let Some(source_id) = ev.source else {
                    continue;
                };
                if let NetworkMessage::Hello {
                    version,
                    installation_id,
                } = &ev.message
                {
                    debug!(
                        "Network: Received Hello (version: {}, install_id: {:?})",
                        version, installation_id
                    );
                    // Find client by associated_id
                    for client in clients.iter_mut() {
                        if client.associated_id == Some(source_id) {
                            // Check if we can re-associate with an existing disconnected entity
                            for (entity, disc_id) in query_disconnected.iter() {
                                if disc_id == &source_id {
                                    debug!(
                                        "Network: Re-associating connection with entity {:?}",
                                        entity
                                    );
                                    commands
                                        .entity(entity)
                                        .remove::<PlayerDisconnected>()
                                        .remove::<Hiding>();
                                }
                            }

                            client.handshake = HandshakeState::Completed;
                            client.needs_full_sync = true;
                            ev_player_joined
                                .write(unnet_core::messages::PlayerJoinedEvent { id: source_id });

                            welcomes_to_send
                                .push((source_id, NetworkMessage::LobbyWelcome { id: source_id }));
                            welcomes_to_send.push((
                                source_id,
                                NetworkMessage::HostStatus {
                                    app_state: *current_app_state.get(),
                                    match_time_elapsed: summary_data.time_taken_secs,
                                    evidences_found: ghost_guess.evidences_found.len() as u32,
                                    repellent_used: repellent_tracker.crafted_count,
                                    player_statuses: vec![], // Will be updated by next broadcast
                                },
                            ));
                            break;
                        }
                    }
                }
            }
        }
        NetworkConn::Client {
            handshake,
            write_queue,
            ..
        } => {
            // Client side automatic Hello
            if matches!(cli.net_mode, NetMode::Join { .. }) && *handshake == HandshakeState::None {
                debug!("Network: Sending Hello...");
                write_queue.push_back(NetworkMessage::Hello {
                    version: "0.1.0".to_string(),
                    installation_id: runtime_installation_id.map(|x| x.0).unwrap_or_default(),
                });
                *handshake = HandshakeState::HelloSent;
            }

            for ev in ev_reader.read() {
                match &ev.message {
                    NetworkMessage::LobbyWelcome { id } => {
                        info!("Network: Received LobbyWelcome (Your ID: {:?})", id);
                        if matches!(cli.net_mode, NetMode::Join { .. }) {
                            local_id.0 = Some(*id);
                            *handshake = HandshakeState::Completed;
                        }
                    }
                    NetworkMessage::Welcome {
                        id,
                        map_seed,
                        map_filepath,
                        difficulty_id,
                    } => {
                        info!(
                            "Network: Received Welcome (Your ID: {:?}, Seed: {}, Map: {})",
                            id, map_seed, map_filepath
                        );
                        if matches!(cli.net_mode, NetMode::Join { .. }) {
                            local_id.0 = Some(*id);
                            *handshake = HandshakeState::Completed;
                            // Apply difficulty
                            if let Ok(d) = Difficulty::from_str(difficulty_id) {
                                *current_difficulty = CurrentDifficulty::new(d);
                            }
                            // Store map path for later loading (after assets are ready)
                            if !map_filepath.is_empty() {
                                info!(
                                    "Network: Will load map '{}' after asset loading completes",
                                    map_filepath
                                );
                                pending_map.map_filepath = Some(map_filepath.clone());
                            } else {
                                warn!("Network: Host sent empty map filepath!");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    for (id, msg) in welcomes_to_send {
        conn.host_send_to(id, msg);
    }

    measure.end_ms();
}

pub(crate) fn client_lobby_state_handler(
    mut ev_reader: MessageReader<NetworkDataEvent>,
    mut lobby_data: ResMut<LobbyData>,
    cli: Res<CliOptions>,
    mut commands: Commands,
) {
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        return;
    }
    for ev in ev_reader.read() {
        if let NetworkMessage::LobbyState {
            players,
            selected_map,
            selected_difficulty,
            room_owner,
        } = &ev.message
        {
            lobby_data.players = players.clone();
            lobby_data.selected_map = selected_map.clone();
            lobby_data.selected_difficulty = selected_difficulty.clone();
            if let Some(id) = room_owner {
                commands.insert_resource(unnet_core::resources::RoomOwner(*id));
            }
        }
    }
}

pub(crate) fn client_start_mission_handler(
    mut ev_reader: MessageReader<NetworkDataEvent>,
    mut pending_map: ResMut<crate::resources::PendingMapLoad>,
    mut current_difficulty: ResMut<CurrentDifficulty>,
    cli: Res<CliOptions>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        return;
    }
    for ev in ev_reader.read() {
        if let NetworkMessage::StartMission {
            map_seed: _,
            map_filepath,
            difficulty_id,
        } = &ev.message
        {
            info!("Network: Received StartMission for map: {}", map_filepath);
            // Apply difficulty
            if let Ok(d) = Difficulty::from_str(difficulty_id) {
                *current_difficulty = CurrentDifficulty::new(d);
            }
            // Store map path for later loading
            pending_map.map_filepath = Some(map_filepath.clone());
            // Transition to Loading
            next_app_state.set(AppState::Loading);
        }
    }
}

pub(crate) fn host_handle_disconnects_system(
    mut ev_disconnect: MessageReader<unnet_core::messages::NetworkDisconnectEvent>,
    query_players: Query<
        (Entity, &NetworkId),
        (
            With<PlayerTag>,
            Without<unplayer_core::components::PlayerDisconnected>,
        ),
    >,
    mut commands: Commands,
    mut lobby_data: ResMut<LobbyData>,
    cli: Res<CliOptions>,
    room_owner: Option<Res<unnet_core::resources::RoomOwner>>,
) {
    let measure = metrics::HOST_HANDLE_DISCONNECTS.time_measure();
    let mut current_room_owner = room_owner.map(|r| r.0);

    for ev in ev_disconnect.read() {
        for (entity, id) in query_players.iter() {
            if id == &ev.id {
                info!("Network: Marking player {:?} as disconnected", id);
                commands
                    .entity(entity)
                    .insert(PlayerDisconnected)
                    .insert(Hiding { hiding_spot: None });
            }
        }
        // Update LobbyData
        if let Some(player) = lobby_data.players.iter_mut().find(|p| p.id == ev.id) {
            player.connected = false;
        }

        // Dedicated server: If RoomOwner disconnected, assign a new one
        if cli.is_headless() && current_room_owner == Some(ev.id) {
            let next_owner = lobby_data
                .players
                .iter()
                .find(|p| p.connected)
                .map(|p| p.id);

            if let Some(new_id) = next_owner {
                commands.insert_resource(unnet_core::resources::RoomOwner(new_id));
                info!("Network: RoomOwner disconnected, new owner: {:?}", new_id);
                current_room_owner = Some(new_id);
            } else {
                commands.remove_resource::<unnet_core::resources::RoomOwner>();
                info!("Network: RoomOwner disconnected, no players left.");
                current_room_owner = None;
            }
        }
    }
    measure.end_ms();
}

pub(crate) fn client_connection_monitor_system(
    conn: Res<NetworkConn>,
    cli: Res<CliOptions>,
    mut game_next_state: ResMut<NextState<GameState>>,
    current_app_state: Res<State<AppState>>,
    mut host_gone: ResMut<HostGone>,
    lobby_data: Res<LobbyData>,
    time: Res<Time>,
    mut grace_timer: Local<Option<f32>>,
) {
    let measure = metrics::CLIENT_CONNECTION_MONITOR.time_measure();
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        host_gone.0 = false;
        *grace_timer = None;
        measure.end_ms();
        return;
    }
    if *current_app_state.get() != AppState::InGame {
        host_gone.0 = false;
        *grace_timer = None;
        measure.end_ms();
        return;
    }

    let host_disconnected = !matches!(*conn, NetworkConn::Client { .. });
    let host_left_mission = lobby_data
        .host_app_state
        .is_some_and(|s| s != AppState::InGame);

    if host_disconnected {
        // TCP disconnect is instant
        if !host_gone.0 {
            warn!("Network: Lost TCP connection to host. Triggering Pause UI.");
            game_next_state.set(GameState::Pause);
            host_gone.0 = true;
        }
        *grace_timer = None;
    } else if host_left_mission {
        // Host left mission has 2s grace period
        let now = time.elapsed_secs();
        let start_time = grace_timer.get_or_insert(now);
        if now - *start_time > 1.0 && !host_gone.0 {
            warn!("Network: Host left mission. Triggering Pause UI.");
            game_next_state.set(GameState::Pause);
            host_gone.0 = true;
        }
    } else {
        host_gone.0 = false;
        *grace_timer = None;
    }
    measure.end_ms();
}

pub(crate) fn autostart_net_game(
    cli: Res<CliOptions>,
    mut lobby_data: ResMut<LobbyData>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let measure = metrics::AUTOSTART_NET_GAME.time_measure();
    if matches!(cli.net_mode, NetMode::Offline) {
        measure.end_ms();
        return;
    }
    // Host and Join are handled. Offline behaves normally.
    match cli.net_mode {
        NetMode::Host { .. } => {
            if let Some(map_filepath) = &cli.map_path {
                info!("Pre-populating lobby with map: {}", map_filepath);
                lobby_data.selected_map = Some(map_filepath.clone());

                if let Some(diff_id) = &cli.difficulty_id {
                    lobby_data.selected_difficulty = diff_id.clone();
                }
            }
            if cli.is_headless() {
                info!("Network: Dedicated server. Entering Lobby.");
                next_app_state.set(AppState::Lobby);
            }
        }
        NetMode::Join { .. } => {
            if cli.is_headless() {
                info!("Network: Autostarting client join. Entering Lobby.");
                next_app_state.set(AppState::Lobby);
            }
        }
        NetMode::Offline => {}
    }

    measure.end_ms();
}

pub(crate) fn headless_summary_reset_system(
    mut next_state: ResMut<NextState<AppState>>,
    cli: Res<CliOptions>,
    time: Res<Time>,
    mut timer: Local<f32>,
    app_state: Res<State<AppState>>,
) {
    if !cli.dedicated {
        return;
    }

    if *app_state.get() != AppState::Summary {
        *timer = 0.0;
        return;
    }

    *timer += time.delta_secs();
    if *timer > 0.5 {
        info!("Headless: Mission summary period ended. Returning to Lobby.");
        next_state.set(AppState::Lobby);
        *timer = 0.0;
    }
}
