use crate::resources::{HandshakeState, LocalPlayerId, NetworkConn};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rand::Rng;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use unbehavior::behavior::{Behavior, Interactive};
use unbehavior::roomdb::RoomDB;
use unbehavior::state::TileState;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::resources::board_topology::{BoardEntityField, BoardTopology};
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unevents_core::events::loadlevel::LoadLevelEvent;
use unevents_core::events::roomchanged::{InteractionExecutionType, RoomChangedEvent};
use unevents_core::events::sound::SoundEvent;
use ungear_core::components::core::Battery;
use ungear_core::components::playergear::PlayerGear;
use ungearitems_core::components::flashlight::Flashlight;
use ungearitems_core::components::sage::{SageSmokeParticle, SmokeParticleTimer};
use unghost_core::components::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use uninteraction_core::interaction::{ExecuteInteractionEvent, Toggleable};
use unnet_core::messages::{
    GearSyncState, GhostState, MapTileState, NetworkDataEvent, NetworkMessage, PlayerGearState,
    PlayerState, RoomSync, TransientEvent,
};
use unnet_core::network_id::NetworkId;
use unplayer_core::components::{Hiding, MainPlayer, PlayerInput, PlayerSprite, Stamina};
use unrender_std::components::animation::{AnimationTimer, CharacterAnimation};
use unrender_std::components::game::GameSprite;
use unrender_std::components::sprite_layer::SpriteLayer;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use untags_core::tags::GhostTag;
use untypes_core::cli::{CliOptions, NetMode};
use untypes_core::difficulty::Difficulty;
use untypes_core::states::{AppState, GameState};

pub fn startup_network_system(
    cli: Res<CliOptions>,
    mut conn: ResMut<NetworkConn>,
    mut local_id: ResMut<LocalPlayerId>,
) {
    match &cli.net_mode {
        NetMode::Offline => {
            *conn = NetworkConn::Disconnected;
            local_id.0 = Some(NetworkId(1));
        }
        NetMode::Host { port } => {
            local_id.0 = Some(NetworkId(1));
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
    mut local_id: ResMut<LocalPlayerId>,
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
                        local_id.0 = Some(id);
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
        &unspatial_core::direction::Direction,
        Option<&Hiding>,
        Option<&PlayerGear>,
        Option<&Stamina>,
        &AnimationTimer,
    )>,
    query_ghosts: Query<
        (&NetworkId, &Position, &GhostSprite, &GhostBehaviorDynamics),
        With<GhostTag>,
    >,
    time: Res<Time>,
    room_db: Res<RoomDB>,
    query_map_tiles: Query<(&Position, &Behavior), With<Interactive>>,
    query_gear: Query<(
        &NetworkId,
        &Position,
        &Toggleable,
        Option<&Battery>,
        Option<&Flashlight>,
        Option<&ungearitems_core::components::sage::SageBundleData>,
        Option<&ungearitems_core::components::repellentflask::RepellentFlask>,
    )>,
    query_net_id: Query<&NetworkId>,
    game_state: Res<State<GameState>>,
    app_state: Res<State<AppState>>,
    mut ev_sound: MessageReader<SoundEvent>,
    mut ev_transient: MessageReader<unnet_core::messages::TransientEvent>,
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
        .map(|(p, pos, dir, hiding, _, stamina, anim)| PlayerState {
            id: p.id,
            position: [pos.x, pos.y, pos.z, f32::atan2(dir.dy, dir.dx)],
            is_hiding: hiding.is_some(),
            stamina: stamina.map(|s| s.current).unwrap_or(100.0),
            is_running: stamina.map(|s| s.running).unwrap_or(false),
            frame: anim.idx() as u16,
        })
        .collect();

    let player_gear = query_players
        .iter()
        .filter_map(|(p, _, _, _, gear, _, _)| {
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
        .map(|(id, pos, ghost, dynamics)| GhostState {
            id: *id,
            position: [pos.x, pos.y, pos.z],
            warp: ghost.warp,
            hunt_warning_active: ghost.hunt_warning_active,
            hunt_warning_intensity: ghost.hunt_warning_intensity,
            calm_time_secs: ghost.calm_time_secs,
            repellent_hits_delta: ghost.repellent_hits_delta,
            repellent_misses_delta: ghost.repellent_misses_delta,
            freezing_temp_clarity: dynamics.freezing_temp_clarity,
            floating_orbs_clarity: dynamics.floating_orbs_clarity,
            uv_ectoplasm_clarity: dynamics.uv_ectoplasm_clarity,
            emf_level5_clarity: dynamics.emf_level5_clarity,
            evp_recording_clarity: dynamics.evp_recording_clarity,
            spirit_box_clarity: dynamics.spirit_box_clarity,
            rl_presence_clarity: dynamics.rl_presence_clarity,
            cpm500_clarity: dynamics.cpm500_clarity,
            visual_alpha_multiplier: dynamics.visual_alpha_multiplier,
            rage_tendency_multiplier: dynamics.rage_tendency_multiplier,
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
        .map(|(id, pos, toggle, battery, flashlight, sage, repellent)| {
            let details = if let Some(f) = flashlight {
                unnet_core::messages::GearDetails::Flashlight(f.status.clone())
            } else if let Some(s) = sage {
                unnet_core::messages::GearDetails::Sage {
                    consumed: s.consumed,
                    is_active: s.is_active,
                    remaining_secs: s.burn_timer.remaining_secs(),
                }
            } else if let Some(r) = repellent {
                unnet_core::messages::GearDetails::RepellentFlask {
                    qty: r.qty,
                    active: r.active,
                }
            } else {
                unnet_core::messages::GearDetails::None
            };
            GearSyncState {
                id: *id,
                position: [pos.x, pos.y, pos.z],
                is_on: toggle.is_on,
                details,
                battery: battery.map(|b| b.level).unwrap_or(0.0),
            }
        })
        .collect();

    let mut events: Vec<unnet_core::messages::TransientEvent> = ev_sound
        .read()
        .map(|ev| unnet_core::messages::TransientEvent::PlaySound {
            sound_file: ev.sound_file.clone(),
            volume: ev.volume,
            position: ev.position.map(|p| [p.x, p.y, p.z]),
        })
        .collect();

    events.extend(ev_transient.read().cloned());

    conn.send(NetworkMessage::Snapshot {
        tick,
        app_state: *app_state.get(),
        game_state: *game_state.get(),
        players,
        ghosts,
        rooms,
        map_tiles,
        gear,
        player_gear,
        events,
    });
}

pub fn client_send_input_system(
    mut conn: ResMut<NetworkConn>,
    cli: Res<CliOptions>,
    local_id: Res<LocalPlayerId>,
    query_player: Query<&PlayerInput, With<MainPlayer>>,
) {
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        return;
    }
    if !conn.is_active() {
        return;
    }

    let Some(player_id) = local_id.0 else {
        return;
    };

    for input in query_player.iter() {
        conn.send(NetworkMessage::PlayerInput {
            player_id,
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
    asset_server: Res<AssetServer>,
    mut ev_reader: MessageReader<NetworkDataEvent>,
    mut query_players: Query<
        (
            Entity,
            &NetworkId,
            &mut Position,
            &mut AnimationTimer,
            &mut unspatial_core::direction::Direction,
            Option<&Hiding>,
            Option<&mut PlayerGear>,
            Option<&MainPlayer>,
            &mut Stamina,
        ),
        (With<PlayerSprite>, Without<GhostTag>, Without<Behavior>),
    >,
    mut query_ghosts: Query<
        (
            &NetworkId,
            &mut Position,
            &mut GhostSprite,
            &mut GhostBehaviorDynamics,
        ),
        (With<GhostTag>, Without<PlayerSprite>, Without<Behavior>),
    >,
    mut ev_room: MessageWriter<RoomChangedEvent>,
    board_field: Res<BoardEntityField>,
    board_topo: Res<BoardTopology>,
    query_tiles: Query<
        (&Position, &Behavior),
        (With<Interactive>, Without<PlayerSprite>, Without<GhostTag>),
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
            Option<&mut ungearitems_core::components::sage::SageBundleData>,
            Option<&mut ungearitems_core::components::repellentflask::RepellentFlask>,
        ),
        (Without<PlayerSprite>, Without<GhostTag>, Without<Behavior>),
    >,
    query_net_entities: Query<(Entity, &NetworkId)>,
    mut ev_sound: MessageWriter<SoundEvent>,
) {
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        return;
    }

    for ev in ev_reader.read() {
        if let NetworkMessage::Snapshot {
            tick: _,
            app_state: server_app_state,
            game_state: server_game_state,
            players,
            ghosts,
            rooms,
            map_tiles,
            gear,
            player_gear,
            events,
        } = &ev.message
        {
            // Sync AppState
            if *server_app_state != *states.current_app_state.get() {
                states.app_next_state.set(*server_app_state);
            }

            // Sync GameState
            if *server_game_state != *states.current_game_state.get() {
                states.game_next_state.set(*server_game_state);
            }

            let net_to_entity: std::collections::HashMap<NetworkId, Entity> =
                query_net_entities.iter().map(|(e, id)| (*id, e)).collect();

            // Update players
            for p_state in players {
                for (
                    p_entity,
                    id,
                    mut pos,
                    mut anim,
                    mut dir,
                    hiding,
                    _,
                    main_player,
                    mut stamina,
                ) in query_players.iter_mut()
                {
                    if *id == p_state.id {
                        let old_pos = *pos;
                        pos.x = p_state.position[0];
                        pos.y = p_state.position[1];
                        pos.z = p_state.position[2];

                        let orientation = p_state.position[3];
                        dir.dx = f32::cos(orientation);
                        dir.dy = f32::sin(orientation);

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

                        if main_player.is_none() {
                            stamina.current = p_state.stamina;
                            stamina.running = p_state.is_running;
                            // We do not want to set the actual frame. there's no need to sync this as the client
                            // is free to play the animation at its own pace. Setting the frame would open us to
                            // sync issues.
                            // anim.set_idx(p_state.frame as usize);
                        }

                        let animation_speed_factor = if p_state.is_running { 1.5 } else { 1.0 };
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
                                CharacterAnimation::from_dir(
                                    dscreen.x * 60.0 * animation_speed_factor,
                                    dscreen.y * 120.0 * animation_speed_factor,
                                )
                                .to_vec(),
                            );
                        } else {
                            let dscreen = perspective::direction_to_screen_coord(*dir);
                            anim.set_range(
                                CharacterAnimation::from_dir(dscreen.x * 0.001, dscreen.y * 0.001)
                                    .to_vec(),
                            );
                        }
                    }
                }
            }

            // Update player gear
            for pg_state in player_gear {
                for (_, id, _, _, _, _, gear, _, _) in query_players.iter_mut() {
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
                for (id, mut pos, mut ghost, mut dynamics) in query_ghosts.iter_mut() {
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

                        dynamics.freezing_temp_clarity = g_state.freezing_temp_clarity;
                        dynamics.floating_orbs_clarity = g_state.floating_orbs_clarity;
                        dynamics.uv_ectoplasm_clarity = g_state.uv_ectoplasm_clarity;
                        dynamics.emf_level5_clarity = g_state.emf_level5_clarity;
                        dynamics.evp_recording_clarity = g_state.evp_recording_clarity;
                        dynamics.spirit_box_clarity = g_state.spirit_box_clarity;
                        dynamics.rl_presence_clarity = g_state.rl_presence_clarity;
                        dynamics.cpm500_clarity = g_state.cpm500_clarity;
                        dynamics.visual_alpha_multiplier = g_state.visual_alpha_multiplier;
                        dynamics.rage_tendency_multiplier = g_state.rage_tendency_multiplier;
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
                for (id, mut pos, mut toggle, battery, flashlight, sage, repellent) in
                    query_gear.iter_mut()
                {
                    if *id == g_sync.id {
                        pos.x = g_sync.position[0];
                        pos.y = g_sync.position[1];
                        pos.z = g_sync.position[2];
                        toggle.is_on = g_sync.is_on;
                        if let Some(mut b) = battery {
                            b.level = g_sync.battery;
                        }

                        match &g_sync.details {
                            unnet_core::messages::GearDetails::Flashlight(status) => {
                                if let Some(mut f) = flashlight {
                                    f.status = status.clone();
                                }
                            }
                            unnet_core::messages::GearDetails::Sage {
                                consumed,
                                is_active,
                                remaining_secs,
                            } => {
                                if let Some(mut s) = sage {
                                    s.consumed = *consumed;
                                    s.is_active = *is_active;

                                    let elapsed =
                                        s.burn_timer.duration().as_secs_f32() - remaining_secs;
                                    s.burn_timer.set_elapsed(std::time::Duration::from_secs_f32(
                                        elapsed.max(0.0),
                                    ));
                                }
                            }
                            unnet_core::messages::GearDetails::RepellentFlask { qty, active } => {
                                if let Some(mut r) = repellent {
                                    r.qty = *qty;
                                    r.active = *active;
                                }
                            }
                            unnet_core::messages::GearDetails::None => {}
                        }
                    }
                }
            }

            // Update events
            for event in events {
                match event {
                    TransientEvent::PlaySound {
                        sound_file,
                        volume,
                        position,
                    } => {
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
                    TransientEvent::SpawnParticle {
                        particle_type,
                        position,
                    } => {
                        if particle_type == "smoke" {
                            let mut rng = unfoundation_core::random_seed::rng();
                            let pos = Position {
                                x: position[0],
                                y: position[1],
                                z: position[2],
                                visual_priority: 0.0,
                            };
                            commands
                                .spawn(Sprite {
                                    image: asset_server.load("img/smoke.png"),
                                    color: Color::NONE,
                                    ..default()
                                })
                                .insert(
                                    Transform::from_translation(perspective::to_screen_coord(pos))
                                        .with_scale(Vec3::new(0.2, 0.2, 0.2)),
                                )
                                .insert(SageSmokeParticle)
                                .insert(GameSprite)
                                .insert(pos)
                                .insert(unspatial_core::direction::Direction {
                                    dx: rng.random_range(-0.9..0.9),
                                    dy: rng.random_range(-0.9..0.9),
                                    dz: rng.random_range(-0.5..0.5),
                                })
                                .insert(MapColor {
                                    color: Color::srgba(1.0, 1.0, 1.0, 0.20),
                                })
                                .insert(SmokeParticleTimer(Timer::from_seconds(
                                    5.0,
                                    TimerMode::Once,
                                )))
                                .insert(SpriteLayer::default());
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
    mut query_players: Query<(&NetworkId, &mut PlayerInput), Without<MainPlayer>>,
    query_van: Query<(&Position, &Behavior)>,
    mut game_next_state: ResMut<NextState<GameState>>,
    query_player_pos: Query<(&NetworkId, &Position), (With<PlayerSprite>, Without<MainPlayer>)>,
) {
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        return;
    }

    for ev in ev_reader.read() {
        match &ev.message {
            NetworkMessage::PlayerInput {
                player_id,
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
                for (id, mut input) in query_players.iter_mut() {
                    if id == player_id {
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
            }
            NetworkMessage::RequestTruckEntry => {
                debug!("Network: Received RequestTruckEntry from client");
                let mut near_van = false;
                for (_id, p_pos) in query_player_pos.iter() {
                    // For now we just check if ANY player (client) is near van when they request it
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
