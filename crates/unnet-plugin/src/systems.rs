use crate::events::NetworkDataEvent;
use crate::resources::{HandshakeState, NetworkConn};
use bevy::prelude::*;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use unbehavior::behavior::{Behavior, Interactive};
use unbehavior::roomdb::RoomDB;
use unbehavior::state::TileState;
use unboard_core::resources::board_topology::{BoardEntityField, BoardTopology};
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unevents_core::events::loadlevel::LoadLevelEvent;
use unevents_core::events::roomchanged::{InteractionExecutionType, RoomChangedEvent};
use uninteraction_core::interaction::Toggleable;
use uninteraction_core::interactivestuff::InteractiveStuff;
use unnet_core::messages::{GearSyncState, MapTileState, NetworkMessage, RoomSync};
use unplayer_core::components::{MainPlayer, PlayerInput, PlayerSprite};
use unrender_std::components::animation::{AnimationTimer, CharacterAnimation};
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use untags_core::tags::{GhostTag, NetworkId};
use untypes_core::cli::{CliOptions, NetMode};
use untypes_core::difficulty::Difficulty;

pub fn startup_network_system(cli: Res<CliOptions>, mut conn: ResMut<NetworkConn>) {
    match &cli.net_mode {
        NetMode::Offline => {
            *conn = NetworkConn::Disconnected;
        }
        NetMode::Host { port } => {
            let addr = format!("0.0.0.0:{}", port);
            match TcpListener::bind(&addr) {
                Ok(listener) => {
                    if let Err(e) = listener.set_nonblocking(true) {
                        error!("Failed to set listener non-blocking: {}", e);
                    } else {
                        info!("Network: Listening on {}", addr);
                        *conn = NetworkConn::Listening(listener);
                    }
                }
                Err(e) => error!("Network: Failed to bind to {}: {}", addr, e),
            }
        }
        NetMode::Join { address } => {
            info!("Network: Connecting to {}...", address);
            match TcpStream::connect(address) {
                Ok(stream) => {
                    if let Err(e) = stream.set_nonblocking(true) {
                        error!("Failed to set stream non-blocking: {}", e);
                    } else {
                        info!("Network: Connected to {}", address);
                        *conn = NetworkConn::Active {
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
}

pub fn network_io_system(
    mut conn: ResMut<NetworkConn>,
    mut ev_writer: MessageWriter<NetworkDataEvent>,
) {
    let mut new_conn = None;

    match &mut *conn {
        NetworkConn::Disconnected => {}
        NetworkConn::Listening(listener) => match listener.accept() {
            Ok((stream, addr)) => {
                info!("Network: Client connected from {}", addr);
                if let Err(e) = stream.set_nonblocking(true) {
                    error!("Failed to set client stream non-blocking: {}", e);
                } else {
                    new_conn = Some(NetworkConn::Active {
                        stream,
                        read_buffer: String::new(),
                        write_queue: VecDeque::new(),
                        handshake: HandshakeState::None,
                    });
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => error!("Network: Accept error: {}", e),
        },
        NetworkConn::Active {
            stream,
            read_buffer,
            write_queue,
            handshake: _,
        } => {
            // --- Read ---
            let mut buf = [0u8; 4096];
            match stream.read(&mut buf) {
                Ok(0) => {
                    info!("Network: Connection closed by peer");
                    *conn = NetworkConn::Disconnected;
                    return;
                }
                Ok(n) => {
                    if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                        read_buffer.push_str(s);
                    }

                    while let Some(pos) = read_buffer.find('\n') {
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
                        *read_buffer = read_buffer[pos + 1..].to_string();
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    error!("Network: Read error: {}", e);
                    *conn = NetworkConn::Disconnected;
                    return;
                }
            }

            // --- Write ---
            while let Some(msg) = write_queue.pop_front() {
                match serde_json::to_string(&msg) {
                    Ok(mut json) => {
                        json.push('\n');
                        if let Err(e) = stream.write_all(json.as_bytes()) {
                            error!("Network: Write error: {}", e);
                            *conn = NetworkConn::Disconnected;
                            return;
                        }
                    }
                    Err(e) => error!("Network: Serialization error: {}", e),
                }
            }
        }
    }

    if let Some(c) = new_conn {
        *conn = c;
    }
}

pub fn handshake_handler_system(
    mut conn: ResMut<NetworkConn>,
    mut ev_reader: MessageReader<NetworkDataEvent>,
    cli: Res<CliOptions>,
) {
    let mut to_send = Vec::new();
    let mut new_handshake = None;

    if let NetworkConn::Active { handshake, .. } = &*conn {
        // Client side automatic Hello
        if matches!(cli.net_mode, NetMode::Join { .. }) && *handshake == HandshakeState::None {
            info!("Network: Sending Hello...");
            to_send.push(NetworkMessage::Hello {
                version: "0.1.0".to_string(),
            });
            new_handshake = Some(HandshakeState::HelloSent);
        }

        let mut events = Vec::new();
        for ev in ev_reader.read() {
            events.push(ev.message.clone());
        }

        for msg in events {
            match msg {
                NetworkMessage::Hello { version } => {
                    info!("Network: Received Hello (version: {})", version);
                    if matches!(cli.net_mode, NetMode::Host { .. }) {
                        info!("Network: Sending Welcome...");
                        let seed = unfoundation_core::random_seed::heavy_rng_seed();
                        to_send.push(NetworkMessage::Welcome {
                            id: 2, // Client is always 2 in MVP
                            map_seed: seed,
                        });
                        new_handshake = Some(HandshakeState::Completed);
                    }
                }
                NetworkMessage::Welcome { id, map_seed } => {
                    info!(
                        "Network: Received Welcome (Your ID: {}, Seed: {})",
                        id, map_seed
                    );
                    if matches!(cli.net_mode, NetMode::Join { .. }) {
                        new_handshake = Some(HandshakeState::Completed);
                    }
                }
                _ => {}
            }
        }
    }

    if let (Some(hs), NetworkConn::Active { handshake, .. }) = (new_handshake, &mut *conn) {
        *handshake = hs;
    }
    for msg in to_send {
        conn.send(msg);
    }
}

pub fn host_send_snapshots_system(
    mut conn: ResMut<NetworkConn>,
    cli: Res<CliOptions>,
    query_players: Query<(&PlayerSprite, &Position)>,
    query_ghosts: Query<&Position, With<GhostTag>>,
    time: Res<Time>,
    room_db: Res<RoomDB>,
    query_map_tiles: Query<(&Position, &Behavior), With<Interactive>>,
    query_gear: Query<(&NetworkId, &Position, &Toggleable)>,
) {
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        return;
    }
    if !conn.is_active() {
        return;
    }

    let tick = (time.elapsed_secs() * 60.0) as u64;
    let players = query_players
        .iter()
        .map(|(p, pos)| unnet_core::messages::PlayerState {
            id: p.id as u64,
            position: [pos.x, pos.y, pos.z, 0.0], // orientation placeholder
        })
        .collect();

    let ghosts = query_ghosts
        .iter()
        .map(|pos| unnet_core::messages::GhostState {
            position: [pos.x, pos.y, pos.z],
        })
        .collect();

    let rooms = room_db
        .room_state
        .iter()
        .map(|(name, state): (&String, &TileState)| RoomSync {
            name: name.clone(),
            state: match state {
                TileState::On => 1,
                _ => 0,
            },
        })
        .collect();

    let map_tiles = query_map_tiles
        .iter()
        .map(|(pos, beh): (&Position, &Behavior)| MapTileState {
            x: pos.x as i32,
            y: pos.y as i32,
            z: pos.z as i32,
            tileset: beh.cfg().tileset.clone(),
            tileuid: beh.cfg().tileuid,
        })
        .collect();

    let gear = query_gear
        .iter()
        .map(
            |(id, pos, toggle): (&NetworkId, &Position, &Toggleable)| GearSyncState {
                id: id.0,
                position: [pos.x, pos.y, pos.z],
                is_on: toggle.is_on,
            },
        )
        .collect();

    conn.send(NetworkMessage::Snapshot {
        tick,
        players,
        ghosts,
        rooms,
        map_tiles,
        gear,
    });
}

pub fn client_send_input_system(
    mut conn: ResMut<NetworkConn>,
    cli: Res<CliOptions>,
    query_player: Query<&PlayerInput, With<MainPlayer>>,
) {
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        return;
    }
    if !conn.is_active() {
        return;
    }

    for input in query_player.iter() {
        conn.send(NetworkMessage::PlayerInput {
            movement: [input.movement.x, input.movement.y],
            run: input.run,
            interact: input.interact,
            grab: input.grab,
            drop: input.drop,
            use_right_hand: input.use_right_hand,
            use_left_hand: input.use_left_hand,
        });
    }
}

pub fn client_apply_snapshots_system(
    cli: Res<CliOptions>,
    mut ev_reader: MessageReader<NetworkDataEvent>,
    mut query_players: Query<
        (&PlayerSprite, &mut Position, &mut AnimationTimer),
        (Without<GhostTag>, Without<NetworkId>),
    >,
    mut query_ghosts: Query<&mut Position, (With<GhostTag>, Without<NetworkId>)>,
    mut ev_room: MessageWriter<RoomChangedEvent>,
    board_field: Res<BoardEntityField>,
    board_topo: Res<BoardTopology>,
    query_tiles: Query<
        (&Position, &Behavior),
        (Without<PlayerSprite>, Without<GhostTag>, Without<NetworkId>),
    >,
    mut interactive_stuff: InteractiveStuff,
    mut query_gear: Query<
        (&NetworkId, &mut Position, &mut Toggleable),
        (Without<PlayerSprite>, Without<GhostTag>),
    >,
) {
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        return;
    }

    for ev in ev_reader.read() {
        if let NetworkMessage::Snapshot {
            players,
            ghosts,
            rooms,
            map_tiles,
            gear,
            ..
        } = &ev.message
        {
            // Update players
            for p_state in players {
                for (p_sprite, mut pos, mut anim) in query_players.iter_mut() {
                    if p_sprite.id as u64 == p_state.id {
                        let old_pos = *pos;
                        pos.x = p_state.position[0];
                        pos.y = p_state.position[1];
                        pos.z = p_state.position[2];

                        let velocity = Vec2::new(pos.x - old_pos.x, pos.y - old_pos.y);
                        if velocity.length_squared() > 0.00001 {
                            let dscreen = perspective::direction_to_screen_coord(
                                unspatial_core::direction::Direction {
                                    dx: velocity.x,
                                    dy: velocity.y,
                                    dz: 0.0,
                                },
                            );
                            anim.set_range(
                                CharacterAnimation::from_dir(dscreen.x * 60.0, dscreen.y * 120.0)
                                    .to_vec(),
                            );
                        } else {
                            anim.set_range(CharacterAnimation::from_dir(0.0, 0.0).to_vec());
                        }
                    }
                }
            }

            // Update ghosts
            for g_state in ghosts {
                if let Ok(mut pos) = query_ghosts.single_mut() {
                    pos.x = g_state.position[0];
                    pos.y = g_state.position[1];
                    pos.z = g_state.position[2];
                }
            }

            // Update rooms
            let mut room_changed = false;
            for r_sync in rooms {
                let new_state = if r_sync.state == 1 {
                    TileState::On
                } else {
                    TileState::Off
                };
                if let Some(state) = interactive_stuff.roomdb.room_state.get_mut(&r_sync.name)
                    && *state != new_state
                {
                    *state = new_state;
                    room_changed = true;
                }
            }
            if room_changed {
                ev_room.write(RoomChangedEvent::default());
            }

            // Update map tiles
            for t_sync in map_tiles {
                let bpos = BoardPosition {
                    x: t_sync.x as i64,
                    y: t_sync.y as i64,
                    z: t_sync.z as i64,
                };
                let rel_x = bpos.x - board_topo.origin.0 as i64;
                let rel_y = bpos.y - board_topo.origin.1 as i64;
                let rel_z = bpos.z - board_topo.origin.2 as i64;

                if rel_x >= 0
                    && rel_y >= 0
                    && rel_z >= 0
                    && rel_x < board_field.0.shape()[0] as i64
                    && rel_y < board_field.0.shape()[1] as i64
                    && rel_z < board_field.0.shape()[2] as i64
                {
                    let entities = &board_field.0[[rel_x as usize, rel_y as usize, rel_z as usize]];
                    for &entity in entities {
                        if let Ok((pos, beh)) = query_tiles.get(entity)
                            && (beh.cfg().tileset != t_sync.tileset
                                || beh.cfg().tileuid != t_sync.tileuid)
                        {
                            interactive_stuff.execute_interaction(
                                entity,
                                pos,
                                None,
                                beh,
                                None, // Map tiles synced this way are usually not room-bound or redundant
                                InteractionExecutionType::ChangeState,
                            );
                        }
                    }
                }
            }

            // Update gear
            for g_sync in gear {
                for (id, mut pos, mut toggle) in query_gear.iter_mut() {
                    if id.0 == g_sync.id {
                        pos.x = g_sync.position[0];
                        pos.y = g_sync.position[1];
                        pos.z = g_sync.position[2];
                        toggle.is_on = g_sync.is_on;
                    }
                }
            }
        }
    }
}

pub fn host_apply_input_system(
    cli: Res<CliOptions>,
    mut ev_reader: MessageReader<NetworkDataEvent>,
    mut query_players: Query<(&PlayerSprite, &mut PlayerInput)>,
) {
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        return;
    }

    for ev in ev_reader.read() {
        if let NetworkMessage::PlayerInput {
            movement,
            run,
            interact,
            grab,
            drop,
            use_right_hand,
            use_left_hand,
        } = &ev.message
        {
            // Client is always ID 2 in this MVP
            for (p_sprite, mut input) in query_players.iter_mut() {
                if p_sprite.id == 2 {
                    input.movement = Vec2::new(movement[0], movement[1]);
                    input.run = *run;
                    input.interact = *interact;
                    input.grab = *grab;
                    input.drop = *drop;
                    input.use_right_hand = *use_right_hand;
                    input.use_left_hand = *use_left_hand;
                }
            }
        }
    }
}

pub fn autostart_net_game(
    cli: Res<CliOptions>,
    mut ev_load_level: MessageWriter<LoadLevelEvent>,
    mut current_difficulty: ResMut<CurrentDifficulty>,
) {
    if matches!(cli.net_mode, NetMode::Offline) {
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
}
