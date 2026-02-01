use crate::resources::{HandshakeState, NetworkConn};
use bevy::ecs::system::SystemParam;
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
use unevents_core::events::sound::SoundEvent;
use ungear_core::components::core::Battery;
use ungear_core::components::playergear::PlayerGear;
use ungearitems_core::components::flashlight::{Flashlight, FlashlightStatus};
use unghost_core::components::ghost_sprite::GhostSprite;
use uninteraction_core::interaction::{ExecuteInteractionEvent, Toggleable};
use unnet_core::messages::{
    GearSyncState, GhostState, MapTileState, NetworkDataEvent, NetworkMessage, PlayerGearState,
    PlayerState, RoomSync,
};
use unnet_core::network_id::NetworkId;
use unplayer_core::components::{Hiding, MainPlayer, PlayerInput, PlayerSprite};
use unrender_std::components::animation::{AnimationTimer, CharacterAnimation};
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use untags_core::tags::GhostTag;
use untypes_core::cli::{CliOptions, NetMode};
use untypes_core::difficulty::Difficulty;
use untypes_core::states::{AppState, GameState};

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
    mut ev_load_level: MessageWriter<LoadLevelEvent>,
    mut current_difficulty: ResMut<CurrentDifficulty>,
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
                            id: NetworkId(2), // Client is always 2 in MVP
                            map_seed: seed,
                            map_filepath: cli.map_path.clone().unwrap_or_default(),
                            difficulty_id: cli
                                .difficulty_id
                                .clone()
                                .unwrap_or("medium".to_string()),
                        });
                        new_handshake = Some(HandshakeState::Completed);
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
                        new_handshake = Some(HandshakeState::Completed);
                        // Apply difficulty
                        if let Ok(d) = Difficulty::from_str(&difficulty_id) {
                            *current_difficulty = CurrentDifficulty::new(d);
                        }
                        // Load Level
                        if !map_filepath.is_empty() {
                            ev_load_level.write(LoadLevelEvent {
                                map_filepath: map_filepath.clone(),
                            });
                        } else {
                            warn!("Network: Host sent empty map filepath!");
                        }
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
    query_players: Query<(
        &PlayerSprite,
        &Position,
        Option<&Hiding>,
        Option<&PlayerGear>,
    )>,
    query_ghosts: Query<(&NetworkId, &Position, &GhostSprite), With<GhostTag>>,
    time: Res<Time>,
    room_db: Res<RoomDB>,
    query_map_tiles: Query<(&Position, &Behavior), With<Interactive>>,
    query_gear: Query<(
        &NetworkId,
        &Position,
        &Toggleable,
        Option<&Battery>,
        Option<&Flashlight>,
    )>,
    query_net_id: Query<&NetworkId>,
    game_state: Res<State<GameState>>,
    app_state: Res<State<AppState>>,
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
        .map(|(p, pos, hiding, _)| PlayerState {
            id: p.id,
            position: [pos.x, pos.y, pos.z, 0.0], // orientation placeholder
            is_hiding: hiding.is_some(),
        })
        .collect();

    let player_gear = query_players
        .iter()
        .filter_map(|(p, _, _, gear)| {
            gear.map(|g| PlayerGearState {
                player_id: p.id,
                left_hand: g.left_hand.and_then(|e| query_net_id.get(e).ok().cloned()),
                right_hand: g.right_hand.and_then(|e| query_net_id.get(e).ok().cloned()),
                inventory: g
                    .inventory
                    .iter()
                    .filter_map(|&e| query_net_id.get(e).ok().cloned())
                    .collect(),
                held_item: g
                    .held_item
                    .as_ref()
                    .and_then(|h| query_net_id.get(h.entity).ok().cloned()),
            })
        })
        .collect();

    let ghosts = query_ghosts
        .iter()
        .map(|(id, pos, ghost)| GhostState {
            id: *id,
            position: [pos.x, pos.y, pos.z],
            warp: ghost.warp,
            hunt_warning_active: ghost.hunt_warning_active,
            hunt_warning_intensity: ghost.hunt_warning_intensity,
            calm_time_secs: ghost.calm_time_secs,
            repellent_hits_delta: ghost.repellent_hits_delta,
            repellent_misses_delta: ghost.repellent_misses_delta,
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
        .map(|(id, pos, toggle, battery, flashlight)| GearSyncState {
            id: *id,
            position: [pos.x, pos.y, pos.z],
            is_on: toggle.is_on,
            mode: flashlight.map(|f| f.status.to_string()),
            battery: battery.map(|b| b.level).unwrap_or(0.0),
        })
        .collect();

    conn.send(NetworkMessage::Snapshot {
        tick,
        app_state: format!("{:?}", app_state.get()),
        game_state: format!("{:?}", game_state.get()),
        players,
        ghosts,
        rooms,
        map_tiles,
        gear,
        player_gear,
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
            inventory_cycle: input.inventory_cycle,
            inventory_swap: input.inventory_swap,
            target_position: input.target_position.map(|v| [v.x, v.y]),
        });
    }
}

#[derive(SystemParam)]
pub struct SnapshotAppStates<'w> {
    pub game_next_state: ResMut<'w, NextState<GameState>>,
    pub current_game_state: Res<'w, State<GameState>>,
    pub current_app_state: Res<'w, State<AppState>>,
    pub app_next_state: ResMut<'w, NextState<AppState>>,
}

pub fn client_apply_snapshots_system(
    mut commands: Commands,
    cli: Res<CliOptions>,
    mut ev_reader: MessageReader<NetworkDataEvent>,
    mut query_players: Query<
        (
            Entity,
            &NetworkId,
            &mut Position,
            &mut AnimationTimer,
            Option<&Hiding>,
            Option<&mut PlayerGear>,
        ),
        (With<PlayerSprite>, Without<GhostTag>),
    >,
    mut query_ghosts: Query<
        (&NetworkId, &mut Position, &mut GhostSprite),
        (With<GhostTag>, Without<PlayerSprite>),
    >,
    mut ev_room: MessageWriter<RoomChangedEvent>,
    board_field: Res<BoardEntityField>,
    board_topo: Res<BoardTopology>,
    query_tiles: Query<
        (&Position, &Behavior),
        (Without<PlayerSprite>, Without<GhostTag>, Without<NetworkId>),
    >,
    mut ev_interaction: MessageWriter<ExecuteInteractionEvent>,
    mut room_db: ResMut<RoomDB>,
    mut states: SnapshotAppStates,
    mut query_gear: Query<
        (
            &NetworkId,
            &mut Position,
            &mut Toggleable,
            Option<&mut Battery>,
            Option<&mut Flashlight>,
        ),
        (Without<PlayerSprite>, Without<GhostTag>),
    >,
    query_net_entities: Query<(Entity, &NetworkId)>,
) {
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        return;
    }

    for ev in ev_reader.read() {
        if let NetworkMessage::Snapshot {
            tick: _,
            app_state: server_app_state_str,
            game_state: server_game_state_str,
            players,
            ghosts,
            rooms,
            map_tiles,
            gear,
            player_gear,
        } = &ev.message
        {
            // Sync AppState
            let current_app_state_str = format!("{:?}", states.current_app_state.get());
            if *server_app_state_str != current_app_state_str {
                let new_state = match server_app_state_str.as_str() {
                    "MainMenu" => AppState::MainMenu,
                    "InGame" => AppState::InGame,
                    "Summary" => AppState::Summary,
                    _ => *states.current_app_state.get(),
                };
                if new_state != *states.current_app_state.get() {
                    states.app_next_state.set(new_state);
                }
            }

            // Sync GameState
            let current_game_state_str = format!("{:?}", states.current_game_state.get());

            if *server_game_state_str != current_game_state_str {
                let new_state = match server_game_state_str.as_str() {
                    "None" => GameState::None,
                    "Truck" => GameState::Truck,
                    "Pause" => GameState::Pause,
                    _ => *states.current_game_state.get(),
                };
                if new_state != *states.current_game_state.get() {
                    states.game_next_state.set(new_state);
                }
            }

            let net_to_entity: std::collections::HashMap<NetworkId, Entity> =
                query_net_entities.iter().map(|(e, id)| (*id, e)).collect();

            // Update players
            for p_state in players {
                for (p_entity, id, mut pos, mut anim, hiding, _) in query_players.iter_mut() {
                    if *id == p_state.id {
                        let old_pos = *pos;
                        pos.x = p_state.position[0];
                        pos.y = p_state.position[1];
                        pos.z = p_state.position[2];

                        // 2.4 Hiding
                        match (p_state.is_hiding, hiding) {
                            (true, None) => {
                                commands
                                    .entity(p_entity)
                                    .insert(Hiding { hiding_spot: None });
                            }
                            (false, Some(_)) => {
                                commands.entity(p_entity).remove::<Hiding>();
                            }
                            _ => {}
                        }

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

            // Update player gear
            for pg_state in player_gear {
                for (_, id, _, _, _, gear) in query_players.iter_mut() {
                    if *id == pg_state.player_id
                        && let Some(mut gear) = gear
                    {
                        gear.left_hand = pg_state
                            .left_hand
                            .and_then(|nid| net_to_entity.get(&nid))
                            .cloned();
                        gear.right_hand = pg_state
                            .right_hand
                            .and_then(|nid| net_to_entity.get(&nid))
                            .cloned();
                        gear.inventory = pg_state
                            .inventory
                            .iter()
                            .filter_map(|nid| net_to_entity.get(nid))
                            .cloned()
                            .collect();
                        gear.held_item = pg_state
                            .held_item
                            .and_then(|nid| net_to_entity.get(&nid))
                            .map(|&entity| ungear_core::components::playergear::HeldObject {
                                entity,
                            });
                    }
                }
            }

            // Update ghosts
            for g_state in ghosts {
                for (id, mut pos, mut ghost) in query_ghosts.iter_mut() {
                    if *id == g_state.id {
                        pos.x = g_state.position[0];
                        pos.y = g_state.position[1];
                        pos.z = g_state.position[2];
                        ghost.warp = g_state.warp;
                        ghost.hunt_warning_active = g_state.hunt_warning_active;
                        ghost.hunt_warning_intensity = g_state.hunt_warning_intensity;
                        ghost.calm_time_secs = g_state.calm_time_secs;
                        ghost.repellent_hits_delta = g_state.repellent_hits_delta;
                        ghost.repellent_misses_delta = g_state.repellent_misses_delta;
                    }
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
                if let Some(state) = room_db
                    .room_state
                    .get_mut(&r_sync.name)
                    .filter(|s| **s != new_state)
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
                        if let Some((_pos, _beh)) =
                            query_tiles.get(entity).ok().filter(|(_, beh)| {
                                beh.cfg().tileset != t_sync.tileset
                                    || beh.cfg().tileuid != t_sync.tileuid
                            })
                        {
                            debug!(
                                "Client: Applying map tile update at {:?} (tileset: {}, tileuid: {})",
                                bpos, t_sync.tileset, t_sync.tileuid
                            );
                            ev_interaction.write(ExecuteInteractionEvent {
                                entity,
                                ietype: InteractionExecutionType::ChangeState,
                                force_tuid: Some(t_sync.tileuid),
                            });
                        }
                    }
                }
            }

            // Update gear
            for g_sync in gear {
                for (id, mut pos, mut toggle, battery, flashlight) in query_gear.iter_mut() {
                    if *id == g_sync.id {
                        pos.x = g_sync.position[0];
                        pos.y = g_sync.position[1];
                        pos.z = g_sync.position[2];
                        toggle.is_on = g_sync.is_on;
                        if let Some(mut b) = battery {
                            b.level = g_sync.battery;
                        }
                        if let (Some(mut f), Some(status)) = (
                            flashlight,
                            g_sync
                                .mode
                                .as_ref()
                                .and_then(|m| m.parse::<FlashlightStatus>().ok()),
                        ) {
                            f.status = status;
                        }
                    }
                }
            }
        }
    }
}

pub fn host_apply_input_system(
    cli: Res<CliOptions>,
    mut ev_reader: MessageReader<NetworkDataEvent>,
    mut query_players: Query<(&PlayerSprite, &mut PlayerInput, &Position), Without<MainPlayer>>,
    query_van: Query<(&Position, &Behavior)>,
    mut game_next_state: ResMut<NextState<GameState>>,
) {
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        return;
    }

    for ev in ev_reader.read() {
        match &ev.message {
            NetworkMessage::PlayerInput {
                movement,
                run,
                interact,
                grab,
                drop,
                use_right_hand,
                use_left_hand,
                inventory_cycle,
                inventory_swap,
                target_position,
            } => {
                // In this MVP, we apply remote input to all sprites that aren't the MainPlayer.
                // This correctly handles the single client without hardcoding IDs.
                for (_, mut input, _) in query_players.iter_mut() {
                    input.movement = Vec2::new(movement[0], movement[1]);
                    input.run = *run;
                    input.interact = *interact;
                    input.grab = *grab;
                    input.drop = *drop;
                    input.use_right_hand = *use_right_hand;
                    input.use_left_hand = *use_left_hand;
                    input.inventory_cycle = *inventory_cycle;
                    input.inventory_swap = *inventory_swap;
                    input.target_position = target_position.map(|v| Vec2::new(v[0], v[1]));
                }
            }
            NetworkMessage::RequestTruckEntry => {
                debug!("Network: Received RequestTruckEntry from client");
                let mut near_van = false;
                for (_, _, p_pos) in query_players.iter() {
                    for (v_pos, v_beh) in query_van.iter() {
                        if v_beh.is_van_entry() && p_pos.delta(*v_pos).distance() < 2.0 {
                            near_van = true;
                            break;
                        }
                    }
                }

                if near_van {
                    info!("Network: RequestTruckEntry validated. Transitioning to Truck state.");
                    game_next_state.set(GameState::Truck);
                } else {
                    warn!("Network: Rejected RequestTruckEntry - player too far from van");
                }
            }
            _ => {}
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
    if matches!(cli.net_mode, NetMode::Join { .. }) {
        // Client waits for Welcome message to load level
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

pub fn host_replicate_sounds_system(
    cli: Res<CliOptions>,
    mut conn: ResMut<NetworkConn>,
    mut ev_reader: MessageReader<SoundEvent>,
) {
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        return;
    }
    if !conn.is_active() {
        return;
    }

    for ev in ev_reader.read() {
        conn.send(NetworkMessage::SoundEvent {
            sound_file: ev.sound_file.clone(),
            volume: ev.volume,
            position: ev.position.map(|p| [p.x, p.y, p.z]),
        });
    }
}

pub fn client_replicate_sounds_system(
    cli: Res<CliOptions>,
    mut ev_reader: MessageReader<NetworkDataEvent>,
    mut ev_sound: MessageWriter<SoundEvent>,
) {
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        return;
    }

    for ev in ev_reader.read() {
        if let NetworkMessage::SoundEvent {
            sound_file,
            volume,
            position,
        } = &ev.message
        {
            ev_sound.write(SoundEvent {
                sound_file: sound_file.clone(),
                volume: *volume,
                position: position.map(|p| Position {
                    x: p[0],
                    y: p[1],
                    z: p[2],
                    visual_priority: 0.0,
                }),
            });
        }
    }
}
