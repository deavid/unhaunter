use crate::resources::{HandshakeState, NetworkConn};
use bevy::prelude::*;
use std::collections::VecDeque;
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
                *conn = NetworkConn::Listening(listeners);
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
                        *conn = NetworkConn::Active {
                            stream,
                            read_buffer: String::new(),
                            write_queue: VecDeque::new(),
                            handshake: HandshakeState::None,
                            installation_id: None,
                            associated_id: None,
                            needs_full_sync: false,
                            host_listeners: Vec::new(),
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
) {
    let measure = metrics::NETWORK_IO.time_measure();
    let mut current_conn = std::mem::replace(&mut *conn, NetworkConn::Disconnected);

    // Process outgoing messages from events
    if let NetworkConn::Active {
        ref mut write_queue,
        ..
    } = current_conn
    {
        for msg in ev_send.read() {
            write_queue.push_back(msg.0.clone());
        }
    }

    match current_conn {
        NetworkConn::Disconnected => {
            *conn = NetworkConn::Disconnected;
        }
        NetworkConn::Listening(listeners) => {
            let mut new_conn = None;
            for listener in &listeners {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        info!("Network: Client connected from {}", addr);
                        if let Err(e) = stream.set_nonblocking(true) {
                            error!("Failed to set client stream non-blocking: {}", e);
                        } else {
                            if let Err(e) = stream.set_nodelay(true) {
                                error!("Failed to set TCP_NODELAY for client: {}", e);
                            }
                            new_conn = Some(stream);
                            break;
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => error!("Network: Accept error: {}", e),
                }
            }
            if let Some(stream) = new_conn {
                *conn = NetworkConn::Active {
                    stream,
                    read_buffer: String::new(),
                    write_queue: VecDeque::new(),
                    installation_id: None,
                    handshake: HandshakeState::None,
                    associated_id: None,
                    needs_full_sync: false,
                    host_listeners: listeners,
                };
            } else {
                *conn = NetworkConn::Listening(listeners);
            }
        }
        NetworkConn::Active {
            stream,
            mut read_buffer,
            installation_id,
            mut write_queue,
            handshake,
            associated_id,
            needs_full_sync,
            host_listeners,
        } => {
            // Gracefully reject new connections while active
            for listener in &host_listeners {
                loop {
                    match listener.accept() {
                        Ok((_, addr)) => {
                            warn!(
                                "Network: Rejecting connection from {} (already connected)",
                                addr
                            );
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(e) => {
                            error!("Network: Accept error (while active): {}", e);
                            break;
                        }
                    }
                }
            }

            let mut closed = false;
            // --- Read ---
            loop {
                let mut buf = [0u8; 65536];
                match (&stream).read(&mut buf) {
                    Ok(0) => {
                        info!("Network: Connection closed by peer");
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
                                ev_writer.write(NetworkDataEvent { message });
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
                            if let Err(e) = (&stream).write_all(json.as_bytes()) {
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
                if let Some(id) = associated_id {
                    ev_disconnect.write(unnet_core::messages::NetworkDisconnectEvent { id });
                }
                if !host_listeners.is_empty() {
                    info!("Network: Re-entering Listening state.");
                    *conn = NetworkConn::Listening(host_listeners);
                } else {
                    *conn = NetworkConn::Disconnected;
                }
            } else {
                *conn = NetworkConn::Active {
                    stream,
                    read_buffer,
                    write_queue,
                    handshake,
                    installation_id,
                    associated_id,
                    needs_full_sync,
                    host_listeners,
                };
            }
        }
    }
    measure.end_ms();
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
    mut player_registry: ResMut<crate::resources::PlayerRegistry>,
    runtime_installation_id: Res<unprofile_core::profile::RuntimeInstallationId>,
) {
    let measure = metrics::HANDSHAKE_HANDLER.time_measure();
    let mut to_send = Vec::new();
    let mut new_handshake = None;
    let mut associate_id = None;
    let mut new_installation_id = None;

    if let NetworkConn::Active { handshake, .. } = &*conn {
        // Client side automatic Hello
        if matches!(cli.net_mode, NetMode::Join { .. }) && *handshake == HandshakeState::None {
            debug!("Network: Sending Hello...");
            to_send.push(NetworkMessage::Hello {
                version: "0.1.0".to_string(),
                installation_id: runtime_installation_id.0,
            });
            new_handshake = Some(HandshakeState::HelloSent);
        }

        let mut events = Vec::new();
        for ev in ev_reader.read() {
            events.push(ev.message.clone());
        }

        for msg in events {
            match msg {
                NetworkMessage::Hello {
                    version,
                    installation_id,
                } => {
                    debug!(
                        "Network: Received Hello (version: {}, install_id: {:?})",
                        version, installation_id
                    );
                    if matches!(cli.net_mode, NetMode::Host { .. }) {
                        let id = player_registry.get_or_assign(installation_id);
                        debug!("Network: Sending Welcome to {:?}...", id);

                        // Check if we can re-associate with an existing disconnected entity
                        for (entity, disc_id) in query_disconnected.iter() {
                            if disc_id == &id {
                                debug!(
                                    "Network: Re-associating connection with entity {:?}",
                                    entity
                                );
                                commands
                                    .entity(entity)
                                    .remove::<unplayer_core::components::PlayerDisconnected>();
                            }
                        }

                        let seed = unfoundation_core::random_seed::heavy_rng_seed();
                        to_send.push(NetworkMessage::Welcome {
                            id,
                            map_seed: seed,
                            map_filepath: cli.map_path.clone().unwrap_or_default(),
                            difficulty_id: cli
                                .difficulty_id
                                .clone()
                                .unwrap_or("medium".to_string()),
                        });
                        new_handshake = Some(HandshakeState::Completed);
                        associate_id = Some(id);
                        new_installation_id = Some(installation_id);
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
                        local_id.0 = Some(id);
                        new_handshake = Some(HandshakeState::Completed);
                        // Apply difficulty
                        if let Ok(d) = Difficulty::from_str(&difficulty_id) {
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

    if let (
        Some(hs),
        NetworkConn::Active {
            handshake,
            associated_id,
            needs_full_sync,
            installation_id,
            ..
        },
    ) = (new_handshake, &mut *conn)
    {
        *handshake = hs;
        if let Some(id) = associate_id {
            *associated_id = Some(id);
            if hs == HandshakeState::Completed {
                *needs_full_sync = true;
            }
        }
        if let Some(iid) = new_installation_id {
            *installation_id = Some(iid);
        }
    }
    for msg in to_send {
        conn.send(msg);
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
    if matches!(*conn, NetworkConn::Disconnected) {
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
