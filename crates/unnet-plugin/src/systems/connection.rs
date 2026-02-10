use crate::resources::{ClientConnection, HandshakeState, NetworkConn, PlayerRegistry};
use bevy::prelude::*;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unevents_core::events::loadlevel::LoadLevelEvent;
use unmetrics_core::metrics::SendMetric;
use unnet_core::messages::{NetworkDataEvent, NetworkMessage};
use unnet_core::network_id::NetworkId;
use unnet_core::resources::{HostGone, LocalPlayer};
use untags_core::tags::PlayerTag;
use untypes_core::cli::{CliOptions, NetMode};
use untypes_core::difficulty::Difficulty;
use untypes_core::states::{AppState, GameState};

use crate::metrics;

pub(crate) fn startup_network_system(
    cli: Res<CliOptions>,
    mut conn: ResMut<NetworkConn>,
    mut local_id: ResMut<LocalPlayer>,
) {
    let measure = metrics::STARTUP_NETWORK_SYSTEM.time_measure();
    match &cli.net_mode {
        NetMode::Offline => {
            *conn = NetworkConn::Disconnected;
            local_id.0 = Some(NetworkId(1));
        }
        NetMode::Host {
            port,
            bind_addresses,
        } => {
            local_id.0 = Some(NetworkId(1));
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
) {
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

            // 3. Read/write each client
            let mut to_remove = Vec::new();
            for (idx, client) in clients.iter_mut().enumerate() {
                let closed = do_client_io(client, &mut ev_writer, &mut player_registry);
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
                let mut buf = [0u8; 65536];
                match (stream).read(&mut buf) {
                    Ok(0) => {
                        info!("Network: Connection closed by host");
                        closed = true;
                        break;
                    }
                    Ok(n) => {
                        if let Ok(s) = std::str::from_utf8(&buf[..n]) {
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
) -> bool {
    let mut closed = false;
    // --- Read ---
    loop {
        let mut buf = [0u8; 65536];
        match (&client.stream).read(&mut buf) {
            Ok(0) => {
                info!("Network: Connection closed by peer");
                closed = true;
                break;
            }
            Ok(n) => {
                if let Ok(s) = std::str::from_utf8(&buf[..n]) {
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
    cli: Res<CliOptions>,
    mut local_id: ResMut<LocalPlayer>,
    mut pending_map: ResMut<crate::resources::PendingMapLoad>,
    mut current_difficulty: ResMut<CurrentDifficulty>,
    mut commands: Commands,
    query_disconnected: Query<
        (Entity, &NetworkId),
        With<unplayer_core::components::PlayerDisconnected>,
    >,
    runtime_installation_id: Res<unprofile_core::profile::RuntimeInstallationId>,
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
                                        .remove::<unplayer_core::components::PlayerDisconnected>();
                                }
                            }

                            client.handshake = HandshakeState::Completed;
                            client.needs_full_sync = true;

                            let seed = unfoundation_core::random_seed::heavy_rng_seed();
                            welcomes_to_send.push((
                                source_id,
                                NetworkMessage::Welcome {
                                    id: source_id,
                                    map_seed: seed,
                                    map_filepath: cli.map_path.clone().unwrap_or_default(),
                                    difficulty_id: cli
                                        .difficulty_id
                                        .clone()
                                        .unwrap_or("medium".to_string()),
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
                    installation_id: runtime_installation_id.0,
                });
                *handshake = HandshakeState::HelloSent;
            }

            for ev in ev_reader.read() {
                if let NetworkMessage::Welcome {
                    id,
                    map_seed,
                    map_filepath,
                    difficulty_id,
                } = &ev.message
                {
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
            }
        }
    }

    for (id, msg) in welcomes_to_send {
        conn.host_send_to(id, msg);
    }

    measure.end_ms();
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
) {
    let measure = metrics::HOST_HANDLE_DISCONNECTS.time_measure();
    for ev in ev_disconnect.read() {
        for (entity, id) in query_players.iter() {
            if id == &ev.id {
                info!("Network: Marking player {:?} as disconnected", id);
                commands
                    .entity(entity)
                    .insert(unplayer_core::components::PlayerDisconnected);
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
) {
    let measure = metrics::CLIENT_CONNECTION_MONITOR.time_measure();
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        host_gone.0 = false;
        measure.end_ms();
        return;
    }
    if *current_app_state.get() != AppState::InGame {
        host_gone.0 = false;
        measure.end_ms();
        return;
    }
    if !matches!(*conn, NetworkConn::Client { .. }) {
        if !host_gone.0 {
            warn!("Network: Lost connection to host. Triggering Pause UI.");
            game_next_state.set(GameState::Pause);
            host_gone.0 = true;
        }
    } else {
        host_gone.0 = false;
    }
    measure.end_ms();
}

pub(crate) fn autostart_net_game(
    cli: Res<CliOptions>,
    mut ev_load_level: MessageWriter<LoadLevelEvent>,
    mut current_difficulty: ResMut<CurrentDifficulty>,
) {
    let measure = metrics::AUTOSTART_NET_GAME.time_measure();
    if matches!(cli.net_mode, NetMode::Offline) {
        measure.end_ms();
        return;
    }
    if matches!(cli.net_mode, NetMode::Join { .. }) {
        measure.end_ms();
        return;
    }

    if let Some(map_filepath) = &cli.map_path {
        info!("Autostarting networked game with map: {}", map_filepath);

        if let Some(diff_id) = &cli.difficulty_id {
            match Difficulty::from_str(diff_id) {
                Ok(d) => {
                    info!("Applying difficulty: {}", d);
                    *current_difficulty = CurrentDifficulty::new(d);
                }
                Err(_) => {
                    warn!("Invalid difficulty ID: {}. Using default.", diff_id);
                    *current_difficulty = CurrentDifficulty::default();
                }
            }
        }

        ev_load_level.write(LoadLevelEvent {
            map_filepath: map_filepath.clone(),
        });
    } else if matches!(cli.net_mode, NetMode::Host { .. }) {
        warn!("NetMode::Host active but no --map provided. Staying in Main Menu.");
    }
    measure.end_ms();
}
