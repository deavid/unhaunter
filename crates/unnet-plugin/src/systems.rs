use crate::resources::{HandshakeState, NetworkConn};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use rand::Rng;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use unassets_core::resources::maps::Maps;
use unbehavior::behavior::{Behavior, Interactive};
use unbehavior::roomdb::RoomDB;
use unbehavior::state::TileState;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::resources::board_topology::{BoardEntityField, BoardTopology};
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unevents_core::events::loadlevel::LoadLevelEvent;
use unevents_core::events::roomchanged::{InteractionExecutionType, RoomStateSyncEvent};
use unevents_core::events::sound::SoundEvent;
use unfoundation_core::types::gear::EquipmentPosition;
use ungear_core::components::core::Battery;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use ungearitems_core::components::flashlight::Flashlight;
use ungearitems_core::components::sage::{SageSmokeParticle, SmokeParticleTimer};
use unghost_core::assets::GhostAssets;
use unghost_core::components::ghost_breach::GhostBreach;
use unghost_core::components::ghost_influence::GhostInfluence;
use unghost_core::components::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unghost_core::resources::ghost_guess::GhostGuess;
use uninteraction_core::interaction::{ExecuteInteractionEvent, Toggleable};
use unmetrics_core::metrics::SendMetric;
use unnet_core::messages::{
    GearSyncState, GhostState, HauntedObjectSync, MapTileState, MovableObjectSync,
    NetworkDataEvent, NetworkMessage, PlayerGearState, PlayerState, RoomSync, SnapshotMsg,
    TransientEvent,
};
use unnet_core::network_id::NetworkId;
use unnet_core::resources::{HostGone, LocalPlayer, MissionEndRequested};
use unplayer_core::assets::PlayerAssets;
use unplayer_core::components::{
    Hiding, MainPlayer, PlayerInput, PlayerSpectating, PlayerSprite, Stamina,
};
use unrender_std::components::animation::{AnimationTimer, CharacterAnimation};
use unrender_std::components::game::{GameSprite, MapTileSprite};
use unrender_std::components::sprite_layer::SpriteLayer;
use unrender_std::components::visuals::{
    AlphaModulator, LightSensitive, ResolutionFactor, ShadowCaster, SpectralInfluence,
    UltravioletSensitive,
};
use unrender_std::materials::CustomMaterial1;
use unrender_std::resources::visibility_data::VisibilityData;
use unrender_std::utils::quadcc::QuadCC;
use unsettings_core::audio::AudioSettings;

use unspatial_core::boardposition::{BoardPosition, MapEntityFieldBPos};
use unspatial_core::components::NetworkOriginalMapPosition;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use unsummary_core::summary::SummaryData;
use untags_core::tags::{GhostTag, PlayerTag};
use untruck_core::components::in_truck::InTruck;
use untruck_core::components::truck_ui_button::TruckUIButton;
use untruck_core::types::repellent_tracker::RepellentCraftTracker;
use untruck_core::types::truck_button::{TruckButtonState, TruckButtonType};
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
                    associated_id,
                    needs_full_sync,
                    host_listeners,
                };
            }
        }
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
) {
    let measure = metrics::HANDSHAKE_HANDLER.time_measure();
    let mut to_send = Vec::new();
    let mut new_handshake = None;
    let mut associate_id = None;

    if let NetworkConn::Active { handshake, .. } = &*conn {
        // Client side automatic Hello
        if matches!(cli.net_mode, NetMode::Join { .. }) && *handshake == HandshakeState::None {
            debug!("Network: Sending Hello...");
            to_send.push(NetworkMessage::Hello {
                version: "0.1.0".to_string(),
                previous_id: local_id.0,
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
                    previous_id,
                } => {
                    debug!(
                        "Network: Received Hello (version: {}, prev_id: {:?})",
                        version, previous_id
                    );
                    if matches!(cli.net_mode, NetMode::Host { .. }) {
                        let id = previous_id.unwrap_or(NetworkId(2)); // Client is always 2 in MVP
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
    }
    for msg in to_send {
        conn.send(msg);
    }
    measure.end_ms();
}

#[derive(SystemParam)]
#[allow(clippy::type_complexity)]
pub(crate) struct HostSnapshotParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub query_players: Query<
        'w,
        's,
        (
            &'static PlayerSprite,
            &'static Position,
            &'static unspatial_core::direction::Direction,
            Option<&'static Hiding>,
            Option<&'static InTruck>,
            Option<&'static PlayerGear>,
            Option<&'static Stamina>,
            &'static AnimationTimer,
            Option<&'static PlayerSpectating>,
            Option<&'static PlayerInput>,
        ),
    >,
    pub query_ghosts: Query<
        'w,
        's,
        (
            &'static NetworkId,
            &'static Position,
            &'static GhostSprite,
            &'static GhostBehaviorDynamics,
        ),
        With<GhostTag>,
    >,
    pub time: Res<'w, Time>,
    pub room_db: Res<'w, RoomDB>,
    pub query_map_tiles: Query<'w, 's, (&'static Position, &'static Behavior), With<Interactive>>,
    pub query_gear: Query<
        'w,
        's,
        (
            &'static NetworkId,
            &'static ungear_core::types::gear::kind::GearKind,
            &'static Position,
            Option<&'static Toggleable>,
            Option<&'static DeployedGear>,
            Option<&'static Battery>,
            Option<&'static Flashlight>,
            Option<&'static ungearitems_core::components::sage::SageBundleData>,
            Option<&'static ungearitems_core::components::repellentflask::RepellentFlask>,
            Option<&'static ungearitems_core::components::thermometer::Thermometer>,
            Option<&'static ungearitems_core::components::emfmeter::EMFMeter>,
            Option<&'static ungearitems_core::components::spiritbox::SpiritBox>,
        ),
    >,
    pub query_net_id: Query<'w, 's, &'static NetworkId>,
    pub game_state: Res<'w, State<GameState>>,
    pub app_state: Res<'w, State<AppState>>,
    pub ghost_guess: Res<'w, GhostGuess>,
    pub summary_data: Res<'w, SummaryData>,
    pub changed_tiles: ResMut<'w, unnet_core::resources::ChangedTiles>,
    pub mission_end_requested: Res<'w, MissionEndRequested>,
    pub repellent_craft_tracker: Res<'w, RepellentCraftTracker>,
    pub query_breach: Query<'w, 's, &'static Position, With<GhostBreach>>,
    pub query_influence:
        Query<'w, 's, (&'static NetworkOriginalMapPosition, &'static GhostInfluence)>,
    pub query_movable: Query<
        'w,
        's,
        (
            Entity,
            Option<&'static NetworkId>,
            Ref<'static, Position>,
            &'static NetworkOriginalMapPosition,
        ),
        With<unbehavior::components::Movable>,
    >,
}

pub(crate) fn host_send_snapshots_system(
    mut conn: ResMut<NetworkConn>,
    cli: Res<CliOptions>,
    mut host_params: HostSnapshotParams,
    mut ev_sound: MessageReader<SoundEvent>,
    mut ev_transient: MessageReader<unnet_core::messages::TransientEvent>,
) {
    let measure = metrics::HOST_SEND_SNAPSHOTS.time_measure();
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        measure.end_ms();
        return;
    }
    if !conn.is_active() {
        measure.end_ms();
        return;
    }

    let tick = (host_params.time.elapsed_secs() * 60.0) as u64;
    let players: Vec<PlayerState> = host_params
        .query_players
        .iter()
        .map(
            |(player, pos, dir, hiding, in_truck, _, stamina, anim, spectating, input)| {
                PlayerState {
                    id: player.id,
                    position: [pos.x, pos.y, pos.z],
                    orientation: [dir.dx, dir.dy],
                    target_position: input.and_then(|i| i.target_position).map(|v| [v.x, v.y]),
                    is_hiding: hiding.is_some(),
                    is_in_truck: in_truck.is_some(),
                    stamina: stamina.map(|s| s.current).unwrap_or(100.0),
                    health: player.health,
                    sanity: player.sanity,
                    is_running: stamina.map(|s| s.running).unwrap_or(false),
                    frame: anim.idx() as u16,
                    is_spectating: spectating.is_some(),
                }
            },
        )
        .collect();

    let player_gear = host_params
        .query_players
        .iter()
        .filter_map(|(p, _, _, _, _, gear, _, _, _, _)| {
            gear.map(|g| PlayerGearState {
                player_id: p.id,
                left_hand: g
                    .left_hand
                    .and_then(|e| host_params.query_net_id.get(e).ok().cloned()),
                right_hand: g
                    .right_hand
                    .and_then(|e| host_params.query_net_id.get(e).ok().cloned()),
                inventory: g
                    .inventory
                    .iter()
                    .filter_map(|&e| host_params.query_net_id.get(e).ok().cloned())
                    .collect(),
                held_item: g
                    .held_item
                    .as_ref()
                    .and_then(|h| host_params.query_net_id.get(h.entity).ok().cloned()),
            })
        })
        .collect();

    let ghosts = host_params
        .query_ghosts
        .iter()
        .map(|(id, pos, ghost, dynamics)| GhostState {
            id: *id,
            position: [pos.x, pos.y, pos.z],
            warp: ghost.warp,
            hunt_warning_active: ghost.hunt_warning_active,
            hunt_warning_intensity: ghost.hunt_warning_intensity,
            hunt_target: ghost.hunt_target,
            calm_time_secs: ghost.calm_time_secs,
            repellent_hits_delta: ghost.repellent_hits_delta,
            repellent_misses_delta: ghost.repellent_misses_delta,
            repellent_hits: ghost.repellent_hits,
            class: ghost.class,
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

    let rooms = host_params
        .room_db
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

    let gear: Vec<_> = host_params
        .query_gear
        .iter()
        .map(
            |(
                id,
                kind,
                pos,
                toggle,
                deployed,
                battery,
                flashlight,
                sage,
                repellent,
                thermometer,
                emfm,
                spiritbox,
            )| {
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
                        liquid_content: r.liquid_content,
                    }
                } else if let Some(t) = thermometer {
                    unnet_core::messages::GearDetails::Thermometer { temp: t.temp }
                } else if let Some(e) = emfm {
                    unnet_core::messages::GearDetails::EMF { level: e.emf }
                } else if let Some(s) = spiritbox {
                    unnet_core::messages::GearDetails::SpiritBox {
                        charge: s.charge,
                        ghost_answer: s.ghost_answer,
                    }
                } else {
                    unnet_core::messages::GearDetails::None
                };

                GearSyncState {
                    id: *id,
                    kind: *kind,
                    position: [pos.x, pos.y, pos.z],
                    is_on: toggle.map(|t| t.is_on).unwrap_or(false),
                    is_deployed: deployed.is_some(),
                    deployed_direction: deployed
                        .map(|d| [d.direction.dx, d.direction.dy])
                        .unwrap_or([0.0, 0.0]),
                    details,
                    battery: battery.map(|b| b.level).unwrap_or(0.0),
                }
            },
        )
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

    let mut is_full_sync = false;
    if let NetworkConn::Active {
        needs_full_sync, ..
    } = &mut *conn
        && *needs_full_sync
    {
        is_full_sync = true;
        *needs_full_sync = false;
    }

    let map_tiles = if is_full_sync {
        host_params.changed_tiles.0.clear();
        host_params
            .query_map_tiles
            .iter()
            .map(|(pos, beh): (&Position, &Behavior)| MapTileState {
                x: pos.x as i32,
                y: pos.y as i32,
                z: pos.z as i32,
                tileset: beh.cfg().tileset.clone(),
                tileuid: beh.cfg().tileuid,
                cvo_key: beh.key_cvo().to_key_string(),
            })
            .collect()
    } else {
        host_params.changed_tiles.0.drain(..).collect()
    };

    let breach_position = host_params
        .query_breach
        .single()
        .ok()
        .map(|pos| [pos.x, pos.y, pos.z]);

    let ghost_type = host_params
        .query_ghosts
        .iter()
        .next()
        .map(|(_, _, ghost, _)| ghost.class);

    let haunted_objects = host_params
        .query_influence
        .iter()
        .map(|(orig, influence)| HauntedObjectSync {
            original_position: [
                orig.position.x as i32,
                orig.position.y as i32,
                orig.position.z as i32,
            ],
            tileset: orig.tileset.clone(),
            tileuid: orig.tileuid,
            influence_type: influence.influence_type,
        })
        .collect();

    let movable_objects =
        host_params
            .query_movable
            .iter()
            .filter_map(|(entity, nid, pos, orig)| {
                let mid = match nid {
                    Some(id) => *id,
                    None => {
                        use rand::Rng;
                        let mut rng = rand::rng();
                        let new_id = NetworkId(rng.random_range(1000..u64::MAX));
                        host_params.commands.entity(entity).insert(new_id);
                        new_id
                    }
                };
                let held_by = host_params.query_players.iter().find_map(
                    |(p, _, _, _, _, gear, _, _, _, _)| {
                        gear.and_then(|g| {
                            g.held_item.as_ref().and_then(|h| {
                                if host_params.query_net_id.get(h.entity).ok() == Some(&mid) {
                                    Some(p.id)
                                } else {
                                    None
                                }
                            })
                        })
                    },
                );
                if pos.is_changed() {
                    debug!(
                        "Host sending moved object: id={:?} orig={:?} cur={:?}",
                        mid, orig.position, *pos
                    );

                    Some(MovableObjectSync {
                        id: mid,
                        original_position: [
                            orig.position.x as i32,
                            orig.position.y as i32,
                            orig.position.z as i32,
                        ],
                        tileset: orig.tileset.clone(),
                        tileuid: orig.tileuid,
                        current_position: [pos.x, pos.y, pos.z],
                        held_by,
                    })
                } else {
                    None
                }
            })
            .collect();

    conn.send(NetworkMessage::Snapshot(Box::new(SnapshotMsg {
        tick,
        is_full_sync,
        app_state: *host_params.app_state.get(),
        game_state: *host_params.game_state.get(),
        can_end_mission: host_params.mission_end_requested.0,
        players,
        ghosts,
        rooms,
        map_tiles,
        gear,
        player_gear,
        events,
        evidences_found: host_params
            .ghost_guess
            .evidences_found
            .iter()
            .cloned()
            .collect(),
        evidences_missing: host_params
            .ghost_guess
            .evidences_missing
            .iter()
            .cloned()
            .collect(),
        ghost_type_guess: host_params.ghost_guess.ghost_type,
        ghosts_discarded: host_params
            .ghost_guess
            .ghosts_discarded
            .iter()
            .cloned()
            .collect(),
        mission_result: Box::new(if *host_params.app_state.get() == AppState::Summary {
            Some(unnet_core::messages::MissionResult {
                time_taken_secs: host_params.summary_data.time_taken_secs,
                ghost_types: host_params.summary_data.ghost_types.clone(),
                repellent_used_amt: host_params.summary_data.repellent_used_amt,
                ghosts_unhaunted: host_params.summary_data.ghosts_unhaunted,
                base_score: host_params.summary_data.base_score,
                difficulty_multiplier: host_params.summary_data.difficulty_multiplier,
                grade_multiplier: host_params.summary_data.grade_multiplier,
                average_sanity: host_params.summary_data.average_sanity,
                player_count: host_params.summary_data.player_count as u32,
                alive_count: host_params.summary_data.alive_count as u32,
                full_score: host_params.summary_data.full_score,
                mission_successful: host_params.summary_data.mission_successful,
                money_earned: host_params.summary_data.money_earned,
                grade_achieved: host_params.summary_data.grade_achieved,
                required_deposit: host_params.summary_data.required_deposit,
                mission_reward_base: host_params.summary_data.mission_reward_base,
                deposit_originally_held: host_params.summary_data.deposit_originally_held,
                deposit_returned_to_bank: host_params.summary_data.deposit_returned_to_bank,
                costs_deducted_from_deposit: host_params.summary_data.costs_deducted_from_deposit,
            })
        } else {
            None
        }),
        repellent_crafted_count: host_params.repellent_craft_tracker.crafted_count,
        breach_position,
        ghost_type,
        haunted_objects,
        movable_objects,
    })));
    measure.end_ms();
}

pub(crate) fn client_send_input_system(
    mut conn: ResMut<NetworkConn>,
    cli: Res<CliOptions>,
    local_id: Res<LocalPlayer>,
    query_player: Query<&PlayerInput, With<MainPlayer>>,
    mut ev_net_data: MessageReader<NetworkDataEvent>,
) {
    let measure = metrics::CLIENT_SEND_INPUT.time_measure();
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        measure.end_ms();
        return;
    }
    if !conn.is_active() {
        measure.end_ms();
        return;
    }

    let Some(player_id) = local_id.0 else {
        measure.end_ms();
        return;
    };

    for input in query_player.iter() {
        conn.send(NetworkMessage::PlayerInput {
            player_id,
            movement: [input.movement.x, input.movement.y],
            run: input.run,
            interact: input.interact,
            use_right_hand: input.use_right_hand,
            use_left_hand: input.use_left_hand,
            target_position: input.target_position.map(|v| [v.x, v.y]),
            aim_direction: [input.aim_direction.x, input.aim_direction.y],
        });
    }

    // Pass through CraftRepellent, Truck entry/exit, and Interaction requests
    for ev in ev_net_data.read() {
        match ev.message {
            // Messages we know we must process:
            NetworkMessage::CraftRepellent { .. }
            | NetworkMessage::RequestTruckEntry { .. }
            | NetworkMessage::RequestTruckExit { .. }
            | NetworkMessage::InteractionRequest { .. } => {
                conn.send(ev.message.clone());
            }
            // Messages that we know we must NOT process:
            NetworkMessage::Snapshot(_) => {}
            // Other messages, we report them just in case:
            _ => {
                trace!("Ignoring message - not sending to the host: {ev:?}");
            }
        }
    }
    measure.end_ms();
}

#[derive(SystemParam)]
pub(crate) struct SnapshotAppStates<'w> {
    pub game_next_state: ResMut<'w, NextState<GameState>>,
    pub current_game_state: Res<'w, State<GameState>>,
    pub current_app_state: Res<'w, State<AppState>>,
    pub app_next_state: ResMut<'w, NextState<AppState>>,
}

#[derive(SystemParam)]
#[allow(clippy::type_complexity)]
pub(crate) struct ClientSnapshotParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub cli: Res<'w, CliOptions>,
    pub asset_server: Res<'w, AssetServer>,
    pub player_assets: Res<'w, PlayerAssets>,
    pub ghost_assets: Res<'w, GhostAssets>,
    pub gear_registry: Res<'w, GearSpawnerRegistry>,
    pub materials1: ResMut<'w, Assets<CustomMaterial1>>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub query_players: Query<
        'w,
        's,
        (
            Entity,
            &'static NetworkId,
            &'static mut Position,
            &'static mut AnimationTimer,
            &'static mut unspatial_core::direction::Direction,
            Option<&'static Hiding>,
            Option<&'static InTruck>,
            Option<&'static mut PlayerGear>,
            Option<&'static MainPlayer>,
            &'static mut Stamina,
            Option<&'static PlayerSpectating>,
            &'static mut PlayerSprite,
            &'static mut PlayerInput,
        ),
        (Without<GhostTag>, Without<Behavior>, Without<GhostBreach>),
    >,
    pub query_ghosts: Query<
        'w,
        's,
        (
            &'static NetworkId,
            &'static mut Position,
            &'static mut GhostSprite,
            &'static mut GhostBehaviorDynamics,
        ),
        (
            With<GhostTag>,
            Without<PlayerSprite>,
            Without<Behavior>,
            Without<GhostBreach>,
        ),
    >,
    pub ev_room_sync: MessageWriter<'w, RoomStateSyncEvent>,
    pub board_field: Res<'w, BoardEntityField>,
    pub board_topo: Res<'w, BoardTopology>,
    pub query_tiles: Query<
        'w,
        's,
        &'static Behavior,
        (
            Without<PlayerSprite>,
            Without<GhostTag>,
            Without<GhostBreach>,
        ),
    >,
    pub ev_interaction: MessageWriter<'w, ExecuteInteractionEvent>,
    pub room_db: ResMut<'w, RoomDB>,
    pub states: SnapshotAppStates<'w>,
    pub query_gear: Query<
        'w,
        's,
        (
            Entity,
            &'static NetworkId,
            &'static mut Position,
            Option<&'static mut Toggleable>,
            Option<&'static mut DeployedGear>,
            Option<&'static mut Battery>,
            Option<&'static mut Flashlight>,
            Option<&'static mut ungearitems_core::components::sage::SageBundleData>,
            Option<&'static mut ungearitems_core::components::repellentflask::RepellentFlask>,
            Option<&'static mut ungearitems_core::components::thermometer::Thermometer>,
            Option<&'static mut ungearitems_core::components::emfmeter::EMFMeter>,
            Option<&'static mut ungearitems_core::components::spiritbox::SpiritBox>,
        ),
        (
            Without<unbehavior::components::Movable>,
            Without<PlayerSprite>,
            Without<GhostTag>,
            Without<Behavior>,
            Without<GhostBreach>,
        ),
    >,
    pub query_net_entities: Query<'w, 's, (Entity, &'static NetworkId)>,
    pub ev_sound: MessageWriter<'w, SoundEvent>,
    pub ghost_guess: ResMut<'w, GhostGuess>,
    pub query_buttons: Query<'w, 's, &'static mut TruckUIButton>,
    pub summary_data: ResMut<'w, SummaryData>,
    pub repellent_craft_tracker: ResMut<'w, RepellentCraftTracker>,
    pub mission_end_requested: ResMut<'w, MissionEndRequested>,
    pub query_breach: Query<
        'w,
        's,
        (
            Entity,
            &'static mut Position,
            Option<&'static mut MapEntityFieldBPos>,
        ),
        (
            With<GhostBreach>,
            Without<PlayerSprite>,
            Without<GhostTag>,
            Without<Behavior>,
        ),
    >,
    pub query_orig_pos: Query<
        'w,
        's,
        (
            Entity,
            &'static NetworkOriginalMapPosition,
            Option<&'static GhostInfluence>,
        ),
    >,
    pub query_movable: Query<
        'w,
        's,
        (
            Entity,
            Option<&'static NetworkId>,
            &'static mut Position,
            &'static NetworkOriginalMapPosition,
            Option<&'static mut EquipmentPosition>,
        ),
        (
            With<unbehavior::components::Movable>,
            Without<PlayerSprite>,
            Without<GhostTag>,
            Without<GhostBreach>,
        ),
    >,
}

fn spawn_breach_locally(params: &mut ClientSnapshotParams, snapshot_pos: [f32; 3]) {
    let breach_img_size = Vec2::new(32.0, 64.0);
    let anchor = unghost_core::assets::GHOST_BREACH_ANCHOR;
    let sprite_anchor = Vec2::new(
        breach_img_size.x * (anchor.x + 0.5),
        breach_img_size.y * (0.5 - anchor.y),
    );
    let mesh_handle = params
        .meshes
        .add(Mesh::from(QuadCC::new(breach_img_size, sprite_anchor)));

    let mut material = CustomMaterial1::from_texture(params.ghost_assets.breach.clone());
    material.data.color = Color::BLACK.with_alpha(0.0).into();
    material.data.y_anchor = anchor.y;
    let material_handle = params.materials1.add(material);

    let pos = Position {
        x: snapshot_pos[0],
        y: snapshot_pos[1],
        z: snapshot_pos[2],
        visual_priority: 0.0,
    };

    params
        .commands
        .spawn(Mesh2d(mesh_handle))
        .insert(MeshMaterial2d(material_handle))
        .insert(pos)
        .insert(GameSprite)
        .insert(MapTileSprite)
        .insert(SpriteLayer(0.01))
        .insert(GhostBreach)
        .insert(LightSensitive {
            exposure_factor: 1.1,
            bias: 0.02,
        })
        .insert(UltravioletSensitive {
            intensity: 1.0,
            color_shift: 1.0,
        })
        .insert(AlphaModulator {
            frequency: 0.92,
            amplitude: 0.5,
        });
}

fn spawn_remote_player(params: &mut ClientSnapshotParams, id: NetworkId) -> Entity {
    let player_rf = 1.0;
    let player_image = params.player_assets.character.clone();
    let sprite_size = Vec2::new(32.0 * player_rf, 32.0 * player_rf);
    let anchor = unplayer_core::assets::PLAYER_ANCHOR;
    let sprite_anchor = Vec2::new(
        sprite_size.x * (anchor.x + 0.5),
        sprite_size.y * (0.5 - anchor.y),
    );
    let src_mesh_handle = params
        .meshes
        .add(Mesh::from(QuadCC::new(sprite_size, sprite_anchor)));

    let mut material = CustomMaterial1::from_texture(player_image.clone());
    material.data.sheet_cols = 16;
    material.data.sheet_rows = 4;
    material.data.sprite_width = 32.0 * player_rf;
    material.data.sprite_height = 32.0 * player_rf;
    material.data.upscale_factor = player_rf;
    material.data.y_anchor = anchor.y;

    let material_handle = params.materials1.add(material);

    let mut ec = params.commands.spawn(Mesh2d(src_mesh_handle.clone()));
    ec.insert(MeshMaterial2d(material_handle))
        .insert(Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(
            1.0 / player_rf,
            1.0 / player_rf,
            1.0 / player_rf,
        )))
        .insert(ResolutionFactor(player_rf))
        .insert(GameSprite)
        .insert(unrender_std::components::game::MapTileSprite)
        .insert(SpriteLayer(0.00001));

    ec.insert(PlayerSprite::new(id, Position::new_i64(0, 0, 0)))
        .insert(id)
        .insert(PlayerInput::default())
        .insert(VisibilityData::default())
        .insert(PlayerTag)
        .insert(ShadowCaster::default())
        .insert(Position::new_i64(0, 0, 0))
        .insert(MapEntityFieldBPos(
            Position::new_i64(0, 0, 0).to_board_position(),
        ))
        .insert(unbehavior::components::Movable)
        .insert(LightSensitive {
            exposure_factor: 1.1,
            bias: 0.01,
        })
        .insert(unspatial_core::direction::Direction::new_right())
        .insert(AnimationTimer::from_range(
            Timer::from_seconds(0.20, TimerMode::Repeating),
            CharacterAnimation::from_dir(0.5, 0.5).to_vec(),
        ))
        .insert(Stamina::default())
        .insert(unnavigation_core::components::waypoint::WaypointQueue::default())
        .insert(PlayerGear::default())
        .insert(MapColor {
            color: Color::WHITE,
        });

    ec.id()
}

fn spawn_remote_gear(params: &mut ClientSnapshotParams, g_sync: &GearSyncState) -> Entity {
    let entity = params
        .gear_registry
        .spawn(&mut params.commands, g_sync.kind);
    params.commands.entity(entity).insert(g_sync.id);

    // Apply immediate state to avoid race conditions (1-frame delay)
    params.commands.entity(entity).insert(Position {
        x: g_sync.position[0],
        y: g_sync.position[1],
        z: g_sync.position[2],
        visual_priority: 0.0,
    });
    params.commands.entity(entity).insert(Toggleable {
        is_on: g_sync.is_on,
    });

    if let unnet_core::messages::GearDetails::Flashlight(status) = &g_sync.details {
        params.commands.entity(entity).insert(Flashlight {
            status: status.clone(),
            ..default()
        });
    }

    if g_sync.is_deployed {
        params
            .commands
            .entity(entity)
            .insert(DeployedGear {
                direction: Vec2::from_array(g_sync.deployed_direction).into(),
            })
            .insert(unbehavior::components::FloorItemCollidable);
    }
    entity
}

pub(crate) fn client_apply_snapshots_system(
    mut ev_reader: MessageReader<NetworkDataEvent>,
    mut params: ClientSnapshotParams,
) {
    let measure = metrics::CLIENT_APPLY_SNAPSHOTS.time_measure();
    if !matches!(params.cli.net_mode, NetMode::Join { .. }) {
        measure.end_ms();
        return;
    }

    for ev in ev_reader.read() {
        if let NetworkMessage::Snapshot(snapshot) = &ev.message {
            let SnapshotMsg {
                tick: _,
                is_full_sync,
                app_state: server_app_state,
                game_state: server_game_state,
                can_end_mission,
                players,
                ghosts,
                rooms,
                map_tiles,
                gear,
                player_gear,
                events,
                evidences_found,
                evidences_missing,
                ghost_type_guess,
                ghosts_discarded,
                mission_result,
                repellent_crafted_count,
                breach_position,
                ghost_type,
                haunted_objects,
                movable_objects,
            } = snapshot.as_ref();

            let is_full_sync = *is_full_sync;
            // Sync AppState - but allow independent Summary transition
            let dominated_by_server_app = matches!(
                server_app_state,
                AppState::Loading | AppState::MainMenu | AppState::Summary
            );
            let local_in_summary = *params.states.current_app_state.get() == AppState::Summary;

            if dominated_by_server_app
                || (!local_in_summary
                    && *server_app_state != *params.states.current_app_state.get())
            {
                params.states.app_next_state.set(*server_app_state);
                // If the server forced us into Summary, we must exit any in-game state (like Truck)
                if *server_app_state == AppState::Summary {
                    params.states.game_next_state.set(GameState::None);
                }
            }

            // Sync GameState - but NOT Truck state (that's per-player)
            // Only sync if Host is in a "global" state that affects everyone
            let dominated_by_server =
                matches!(server_game_state, GameState::Pause | GameState::NpcHelp);
            let local_in_truck = *params.states.current_game_state.get() == GameState::Truck;
            let server_in_truck = *server_game_state == GameState::Truck;

            if dominated_by_server
                || (!local_in_truck
                    && !server_in_truck
                    && *server_game_state != *params.states.current_game_state.get())
            {
                // Only sync if we're not locally in the truck
                // This allows Client to stay in Truck while Host is in None
                params.states.game_next_state.set(*server_game_state);
            }
            // If local_in_truck is true, we keep our local Truck state

            // Sync MissionEndRequested
            params.mission_end_requested.0 = *can_end_mission;

            // Sync repellent craft count
            params.repellent_craft_tracker.crafted_count = *repellent_crafted_count;

            // Sync breach position
            match breach_position {
                Some(snapshot_pos) => {
                    if let Ok((_entity, mut pos, _o_bpos)) = params.query_breach.single_mut() {
                        pos.x = snapshot_pos[0];
                        pos.y = snapshot_pos[1];
                        pos.z = snapshot_pos[2];
                    } else if !is_full_sync {
                        spawn_breach_locally(&mut params, *snapshot_pos);
                    }
                }
                None => {
                    for (entity, _, _) in params.query_breach.iter() {
                        params.commands.entity(entity).despawn();
                    }
                }
            }

            // Sync ghost type
            if let Some(t) = ghost_type {
                for (_, _, mut ghost, _) in params.query_ghosts.iter_mut() {
                    if ghost.class != *t {
                        debug!(
                            "Client: Correcting ghost type from {:?} to {:?}",
                            ghost.class, t
                        );
                        ghost.class = *t;
                    }
                }
            }

            // Sync haunted objects
            let mut snap_haunted_entities = std::collections::HashSet::new();

            // Build a lookup map for the client's current entities by their original position.
            // This is O(N) where N is total map entities.
            let mut orig_pos_lookup = std::collections::HashMap::new();
            for (entity, orig, _influence) in params.query_orig_pos.iter() {
                orig_pos_lookup.insert(
                    (
                        orig.position.x as i32,
                        orig.position.y as i32,
                        orig.position.z as i32,
                        orig.tileset.clone(),
                        orig.tileuid,
                    ),
                    entity,
                );
            }

            for haunt_sync in haunted_objects {
                let key = (
                    haunt_sync.original_position[0],
                    haunt_sync.original_position[1],
                    haunt_sync.original_position[2],
                    haunt_sync.tileset.clone(),
                    haunt_sync.tileuid,
                );

                if let Some(&entity) = orig_pos_lookup.get(&key) {
                    snap_haunted_entities.insert(entity);
                    let needs_update = match params.query_orig_pos.get(entity) {
                        Ok((_, _, Some(inf))) => inf.influence_type != haunt_sync.influence_type,
                        _ => true,
                    };
                    if needs_update {
                        params.commands.entity(entity).insert((
                            GhostInfluence {
                                influence_type: haunt_sync.influence_type,
                                charge_value: 0.0,
                            },
                            SpectralInfluence::default(),
                        ));
                    }
                }
            }

            // Remove influence from objects that the host says are not haunted
            for (entity, _, influence) in params.query_orig_pos.iter() {
                if influence.is_some() && !snap_haunted_entities.contains(&entity) {
                    params
                        .commands
                        .entity(entity)
                        .remove::<GhostInfluence>()
                        .remove::<SpectralInfluence>();
                }
            }

            // Sync movable objects
            let mut mov_orig_pos_lookup = std::collections::HashMap::new();
            for (entity, _mid, _pos, orig, _eq) in params.query_movable.iter() {
                let key = (
                    orig.position.x as i32,
                    orig.position.y as i32,
                    orig.position.z as i32,
                    orig.tileset.clone(),
                    orig.tileuid,
                );
                mov_orig_pos_lookup.insert(key, entity);
            }

            for mov_sync in movable_objects {
                let key = (
                    mov_sync.original_position[0],
                    mov_sync.original_position[1],
                    mov_sync.original_position[2],
                    mov_sync.tileset.clone(),
                    mov_sync.tileuid,
                );

                if let Some(&entity) = mov_orig_pos_lookup.get(&key)
                    && let Ok((_entity, id, mut pos, _orig, _eq_pos)) =
                        params.query_movable.get_mut(entity)
                {
                    if id.is_none() {
                        params.commands.entity(entity).insert(mov_sync.id);
                    }
                    pos.x = mov_sync.current_position[0];
                    pos.y = mov_sync.current_position[1];
                    pos.z = mov_sync.current_position[2];

                    match mov_sync.held_by {
                        Some(_) => {
                            params
                                .commands
                                .entity(entity)
                                .remove::<unbehavior::components::FloorItemCollidable>();
                        }
                        None => {
                            params
                                .commands
                                .entity(entity)
                                .insert(unbehavior::components::FloorItemCollidable);
                        }
                    }
                } else {
                    warn!(
                        "Client: Could not find movable object for sync key {:?}",
                        key
                    );
                }
            }

            let mut net_to_entity: std::collections::HashMap<NetworkId, Entity> = params
                .query_net_entities
                .iter()
                .map(|(e, id)| (*id, e))
                .collect();

            let mut seen_ids = std::collections::HashSet::new();
            for p in players {
                seen_ids.insert(p.id);
            }
            for g in gear {
                seen_ids.insert(g.id);
            }
            for g in ghosts {
                seen_ids.insert(g.id);
            }
            for (entity, id) in params.query_net_entities.iter() {
                if !seen_ids.contains(id) {
                    let is_main = params
                        .query_players
                        .get(entity)
                        .map(|q| q.8.is_some())
                        .unwrap_or(false);
                    if !is_main {
                        params.commands.entity(entity).despawn();
                    }
                }
            }

            // Update players
            for p_state in players {
                let p_entity = if let Some(e) = net_to_entity.get(&p_state.id) {
                    *e
                } else {
                    debug!("Spawning remote player {:?}", p_state.id);
                    let ent = spawn_remote_player(&mut params, p_state.id);
                    net_to_entity.insert(p_state.id, ent);
                    ent
                };

                if let Ok((
                    _p_entity,
                    _id,
                    mut pos,
                    mut anim,
                    mut dir,
                    hiding,
                    in_truck,
                    _,
                    main_player,
                    mut stamina,
                    spectating,
                    mut player_sprite,
                    mut player_input,
                )) = params.query_players.get_mut(p_entity)
                {
                    let old_pos = *pos;
                    pos.x = p_state.position[0];
                    pos.y = p_state.position[1];
                    pos.z = p_state.position[2];

                    if main_player.is_none() {
                        dir.dx = p_state.orientation[0];
                        dir.dy = p_state.orientation[1];
                    }

                    // Spectating
                    if p_state.is_spectating && spectating.is_none() {
                        params.commands.entity(p_entity).insert(PlayerSpectating);
                    } else if !p_state.is_spectating && spectating.is_some() {
                        params
                            .commands
                            .entity(p_entity)
                            .remove::<PlayerSpectating>();
                    }

                    // 2.4 Hiding
                    match (p_state.is_hiding, hiding) {
                        (true, None) => {
                            params
                                .commands
                                .entity(p_entity)
                                .insert(Hiding { hiding_spot: None });
                        }
                        (false, Some(_)) => {
                            params.commands.entity(p_entity).remove::<Hiding>();
                        }
                        _ => {}
                    }

                    // InTruck visuals for remote players
                    if main_player.is_none() {
                        match (p_state.is_in_truck, in_truck) {
                            (true, None) => {
                                params
                                    .commands
                                    .entity(p_entity)
                                    .insert(InTruck)
                                    .insert(Hiding { hiding_spot: None });
                            }
                            (false, Some(_)) => {
                                params
                                    .commands
                                    .entity(p_entity)
                                    .remove::<InTruck>()
                                    .remove::<Hiding>();
                            }
                            _ => {}
                        }
                    }

                    // Sync Health & Sanity
                    // This ensures the client sees damage and sanity drain on the main player
                    player_sprite.health = p_state.health;
                    player_sprite.sanity = p_state.sanity;
                    if main_player.is_none() {
                        player_input.target_position =
                            p_state.target_position.map(|t| Vec2::new(t[0], t[1]));
                    }

                    if main_player.is_none() {
                        stamina.current = p_state.stamina;
                        stamina.running = p_state.is_running;
                        // No need to sync frame directly as it causes jitter with local animation timer.
                        // The range and stamina.running are enough for local reproduction.
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
                        let dscreen =
                            perspective::direction_to_screen_coord(*dir).normalize_or_zero();
                        anim.set_range(
                            CharacterAnimation::from_dir(dscreen.x * 0.5, dscreen.y * 0.5).to_vec(),
                        );
                    }
                }
            }

            // Update gear
            for g_sync in gear {
                let g_entity = if let Some(e) = net_to_entity.get(&g_sync.id) {
                    *e
                } else {
                    debug!(
                        "Spawning remote gear {:?} (kind: {:?})",
                        g_sync.id, g_sync.kind
                    );
                    let ent = spawn_remote_gear(&mut params, g_sync);
                    net_to_entity.insert(g_sync.id, ent);
                    ent
                };

                if let Ok((
                    g_entity,
                    _,
                    mut pos,
                    toggle,
                    deployed,
                    battery,
                    flashlight,
                    sage,
                    repellent,
                    thermometer,
                    emf_meter,
                    spiritbox,
                )) = params.query_gear.get_mut(g_entity)
                {
                    pos.x = g_sync.position[0];
                    pos.y = g_sync.position[1];
                    pos.z = g_sync.position[2];
                    if let Some(mut t) = toggle {
                        t.is_on = g_sync.is_on;
                    }

                    match (g_sync.is_deployed, deployed) {
                        (true, None) => {
                            params
                                .commands
                                .entity(g_entity)
                                .insert(DeployedGear {
                                    direction: Vec2::from_array(g_sync.deployed_direction).into(),
                                })
                                .insert(unbehavior::components::FloorItemCollidable);
                        }
                        (true, Some(mut d)) => {
                            d.direction = Vec2::from_array(g_sync.deployed_direction).into();
                        }
                        (false, Some(_)) => {
                            params
                                .commands
                                .entity(g_entity)
                                .remove::<DeployedGear>()
                                .remove::<unbehavior::components::FloorItemCollidable>()
                                .remove::<Sprite>()
                                .remove::<Transform>()
                                .remove::<Visibility>()
                                .remove::<unrender_std::components::game::GameSprite>()
                                .remove::<unrender_std::components::sprite_layer::SpriteLayer>()
                                .remove::<MapColor>();
                        }
                        _ => {}
                    }

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
                        unnet_core::messages::GearDetails::RepellentFlask {
                            qty,
                            active,
                            liquid_content,
                        } => {
                            if let Some(mut r) = repellent {
                                r.qty = *qty;
                                r.active = *active;
                                r.liquid_content = *liquid_content;
                            }
                        }
                        unnet_core::messages::GearDetails::Thermometer { temp } => {
                            if let Some(mut t) = thermometer {
                                t.temp = *temp;
                            }
                        }
                        unnet_core::messages::GearDetails::EMF { level } => {
                            if let Some(mut e) = emf_meter {
                                e.emf = *level;
                                e.emf_level =
                                    ungearitems_core::components::emfmeter::EMFLevel::from_milligauss(
                                        e.emf,
                                    );
                            }
                        }
                        unnet_core::messages::GearDetails::SpiritBox {
                            charge,
                            ghost_answer,
                        } => {
                            if let Some(mut s) = spiritbox {
                                s.charge = *charge;
                                s.ghost_answer = *ghost_answer;
                            }
                        }
                        unnet_core::messages::GearDetails::None => {}
                    }
                }
            }

            // Update player gear
            for pg_state in player_gear {
                for (_, id, _, _, _, _, _, gear, _, _, _, _, _) in params.query_players.iter_mut() {
                    if *id == pg_state.player_id
                        && let Some(mut gear) = gear
                    {
                        let old_left = gear.left_hand;
                        let old_right = gear.right_hand;
                        let old_held = gear.held_item.as_ref().map(|h| h.entity);

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
                        let new_held = gear.held_item.as_ref().map(|h| h.entity);

                        // Derivation: Update EquipmentPosition on referenced gear
                        if let Some(e) = gear.left_hand {
                            params.commands.entity(e).insert(EquipmentPosition::Hand(
                                unfoundation_core::types::gear::Hand::Left,
                            ));
                        }
                        if let Some(e) = gear.right_hand {
                            params.commands.entity(e).insert(EquipmentPosition::Hand(
                                unfoundation_core::types::gear::Hand::Right,
                            ));
                        }
                        for e in &gear.inventory {
                            params.commands.entity(*e).insert(EquipmentPosition::Stowed);
                        }

                        if old_left != gear.left_hand
                            || old_right != gear.right_hand
                            || old_held != new_held
                        {
                            debug!(
                                "Player {:?} gear state: left={:?}, right={:?}, inv_count={}",
                                id,
                                gear.left_hand,
                                gear.right_hand,
                                gear.inventory.len()
                            );
                        }
                    }
                }
            }

            // Update ghosts
            for g_state in ghosts {
                for (id, mut pos, mut ghost, mut dynamics) in params.query_ghosts.iter_mut() {
                    if *id == g_state.id {
                        pos.x = g_state.position[0];
                        pos.y = g_state.position[1];
                        pos.z = g_state.position[2];
                        ghost.warp = g_state.warp;
                        ghost.hunt_warning_active = g_state.hunt_warning_active;
                        ghost.hunt_warning_intensity = g_state.hunt_warning_intensity;
                        ghost.hunt_target = g_state.hunt_target;
                        ghost.calm_time_secs = g_state.calm_time_secs;
                        ghost.repellent_hits_delta = g_state.repellent_hits_delta;
                        ghost.repellent_misses_delta = g_state.repellent_misses_delta;
                        ghost.repellent_hits = g_state.repellent_hits;
                        ghost.class = g_state.class;

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
                if let Some(state) = params
                    .room_db
                    .room_state
                    .get_mut(&r_sync.name)
                    .filter(|s| **s != new_state)
                {
                    *state = new_state;
                    room_changed = true;
                }
            }
            if room_changed {
                params.ev_room_sync.write(RoomStateSyncEvent);
            }

            // Update map tiles
            for t_sync in map_tiles {
                let bpos = BoardPosition {
                    x: t_sync.x as i64,
                    y: t_sync.y as i64,
                    z: t_sync.z as i64,
                };
                let rel_x = bpos.x;
                let rel_y = bpos.y;
                let rel_z = bpos.z;
                let shape = params.board_field.0.shape();

                if rel_x >= 0
                    && rel_y >= 0
                    && rel_z >= 0
                    && rel_x < shape[0] as i64
                    && rel_y < shape[1] as i64
                    && rel_z < shape[2] as i64
                {
                    let entities =
                        &params.board_field.0[[rel_x as usize, rel_y as usize, rel_z as usize]];
                    if entities.is_empty() {
                        debug!(
                            "Client: No entities found at board position {:?} (Rel: {:?}, Board shape: {:?}, Origin: {:?})",
                            bpos,
                            (rel_x, rel_y, rel_z),
                            shape,
                            params.board_topo.origin
                        );
                    }
                    for &entity in entities {
                        if params
                            .query_tiles
                            .get(entity)
                            .ok()
                            .filter(|beh| {
                                // Filter: Is this the correct sprite?
                                let key_cvo = beh.key_cvo().to_key_string();
                                if key_cvo != t_sync.cvo_key {
                                    return false;
                                }
                                // Filter: Has this sprite changed? Does it require a change?
                                beh.cfg().tileset != t_sync.tileset
                                    || beh.cfg().tileuid != t_sync.tileuid
                            })
                            .is_some()
                        {
                            debug!(
                                "Client: Applying map tile update at {:?} (tileset: {}, tileuid: {})",
                                bpos, t_sync.tileset, t_sync.tileuid
                            );
                            params.ev_interaction.write(ExecuteInteractionEvent {
                                entity,
                                ietype: InteractionExecutionType::ChangeState,
                                force_tuid: Some(t_sync.tileuid),
                            });
                        }
                    }
                } else {
                    debug!(
                        "Client: Map tile update out of bounds: {:?} (Rel: {:?}, Board shape: {:?}, Origin: {:?})",
                        bpos,
                        (rel_x, rel_y, rel_z),
                        shape,
                        params.board_topo.origin
                    );
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
                        params.ev_sound.write(SoundEvent {
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
                            params
                                .commands
                                .spawn(Sprite {
                                    image: params.asset_server.load("img/smoke.png"),
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

            // Sync GhostGuess
            let ghost_guess_changed = is_full_sync
                || params.ghost_guess.ghost_type != *ghost_type_guess
                || params.ghost_guess.evidences_found.len() != evidences_found.len()
                || params.ghost_guess.evidences_missing.len() != evidences_missing.len()
                || params.ghost_guess.ghosts_discarded.len() != ghosts_discarded.len()
                || !evidences_found
                    .iter()
                    .all(|e| params.ghost_guess.evidences_found.contains(e))
                || !evidences_missing
                    .iter()
                    .all(|e| params.ghost_guess.evidences_missing.contains(e))
                || !ghosts_discarded
                    .iter()
                    .all(|g| params.ghost_guess.ghosts_discarded.contains(g));

            if ghost_guess_changed {
                params.ghost_guess.ghost_type = *ghost_type_guess;
                params.ghost_guess.evidences_found = evidences_found.iter().cloned().collect();
                params.ghost_guess.evidences_missing = evidences_missing.iter().cloned().collect();
                params.ghost_guess.ghosts_discarded = ghosts_discarded.iter().cloned().collect();

                // Sync TruckUIButtons
                for mut button in params.query_buttons.iter_mut() {
                    match button.class {
                        TruckButtonType::Evidence(e) => {
                            if params.ghost_guess.evidences_found.contains(&e) {
                                button.status = TruckButtonState::Pressed;
                            } else if params.ghost_guess.evidences_missing.contains(&e) {
                                button.status = TruckButtonState::Discard;
                            } else {
                                button.status = TruckButtonState::Off;
                            }
                        }
                        TruckButtonType::Ghost(gt) => {
                            if params.ghost_guess.ghost_type == Some(gt) {
                                button.status = TruckButtonState::Pressed;
                            } else {
                                button.status = TruckButtonState::Off;
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Sync Mission Result
            if let Some(res) = &**mission_result {
                params.summary_data.time_taken_secs = res.time_taken_secs;
                params.summary_data.ghost_types = res.ghost_types.clone();
                params.summary_data.repellent_used_amt = res.repellent_used_amt;
                params.summary_data.ghosts_unhaunted = res.ghosts_unhaunted;
                params.summary_data.base_score = res.base_score;
                params.summary_data.difficulty_multiplier = res.difficulty_multiplier;
                params.summary_data.grade_multiplier = res.grade_multiplier;
                params.summary_data.average_sanity = res.average_sanity;
                params.summary_data.player_count = res.player_count as usize;
                params.summary_data.alive_count = res.alive_count as usize;
                params.summary_data.full_score = res.full_score;
                params.summary_data.mission_successful = res.mission_successful;
                params.summary_data.money_earned = res.money_earned;
                params.summary_data.grade_achieved = res.grade_achieved;
                params.summary_data.required_deposit = res.required_deposit;
                params.summary_data.mission_reward_base = res.mission_reward_base;
                params.summary_data.deposit_originally_held = res.deposit_originally_held;
                params.summary_data.deposit_returned_to_bank = res.deposit_returned_to_bank;
                params.summary_data.costs_deducted_from_deposit = res.costs_deducted_from_deposit;
            }
        } else if let NetworkMessage::MissionSummary { result } = &ev.message {
            params.summary_data.time_taken_secs = result.time_taken_secs;
            params.summary_data.ghost_types = result.ghost_types.clone();
            params.summary_data.repellent_used_amt = result.repellent_used_amt;
            params.summary_data.ghosts_unhaunted = result.ghosts_unhaunted;
            params.summary_data.base_score = result.base_score;
            params.summary_data.difficulty_multiplier = result.difficulty_multiplier;
            params.summary_data.grade_multiplier = result.grade_multiplier;
            params.summary_data.average_sanity = result.average_sanity;
            params.summary_data.player_count = result.player_count as usize;
            params.summary_data.alive_count = result.alive_count as usize;
            params.summary_data.full_score = result.full_score;
            params.summary_data.mission_successful = result.mission_successful;
            params.summary_data.money_earned = result.money_earned;
            params.summary_data.grade_achieved = result.grade_achieved;
            params.summary_data.required_deposit = result.required_deposit;
            params.summary_data.mission_reward_base = result.mission_reward_base;
            params.summary_data.deposit_originally_held = result.deposit_originally_held;
            params.summary_data.deposit_returned_to_bank = result.deposit_returned_to_bank;
            params.summary_data.costs_deducted_from_deposit = result.costs_deducted_from_deposit;

            // Force state transition to Summary, as Host stops sending snapshots once it enters Summary
            params.states.app_next_state.set(AppState::Summary);
            params.states.game_next_state.set(GameState::None);
        }
    }
    measure.end_ms();
}

#[derive(SystemParam)]
#[allow(clippy::type_complexity)]
pub(crate) struct HostApplyInputParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub cli: Res<'w, CliOptions>,
    pub network_conn: Option<ResMut<'w, NetworkConn>>,
    pub ev_reader: MessageReader<'w, 's, NetworkDataEvent>,
    pub query_players: Query<
        'w,
        's,
        (
            Entity,
            &'static NetworkId,
            &'static mut PlayerInput,
            &'static mut PlayerGear,
        ),
        Without<MainPlayer>,
    >,
    pub query_van: Query<'w, 's, (&'static Position, &'static Behavior)>,
    pub query_player_pos: Query<
        'w,
        's,
        (Entity, &'static NetworkId, &'static Position, Has<InTruck>),
        (With<PlayerSprite>, Without<MainPlayer>),
    >,
    pub ev_interaction: MessageWriter<'w, ExecuteInteractionEvent>,
    pub board_field: Res<'w, BoardEntityField>,
    pub craft_tracker: ResMut<'w, RepellentCraftTracker>,
    pub gear_registry: Res<'w, GearSpawnerRegistry>,
    pub q_repellent:
        Query<'w, 's, &'static mut ungearitems_core::components::repellentflask::RepellentFlask>,
    pub q_gearkind: Query<'w, 's, &'static ungear_core::types::gear::kind::GearKind>,
    pub query_net_id: Query<'w, 's, &'static NetworkId>,
    pub asset_server: Res<'w, AssetServer>,
    pub audio_settings: Res<'w, Persistent<AudioSettings>>,
    pub ev_mission: MessageWriter<'w, unevents_core::events::mission::MissionEvent>,
    pub mission_end_requested: Res<'w, unnet_core::resources::MissionEndRequested>,
}

pub(crate) fn host_apply_input_system(mut params: HostApplyInputParams) {
    let measure = metrics::HOST_APPLY_INPUT.time_measure();
    if !matches!(params.cli.net_mode, NetMode::Host { .. }) {
        measure.end_ms();
        return;
    }

    for ev in params.ev_reader.read() {
        match &ev.message {
            NetworkMessage::PlayerInput {
                player_id,
                movement,
                run,
                interact,
                use_right_hand,
                use_left_hand,
                target_position,
                aim_direction,
            } => {
                let mut found_player = false;
                for (_entity, id, mut input, _) in params.query_players.iter_mut() {
                    if id == player_id {
                        found_player = true;
                        input.movement = Vec2::new(movement[0], movement[1]);
                        input.run = *run;
                        input.interact = *interact;
                        input.use_right_hand = *use_right_hand;
                        input.use_left_hand = *use_left_hand;
                        input.target_position = target_position.map(|v| Vec2::new(v[0], v[1]));
                        input.aim_direction = Vec2::new(aim_direction[0], aim_direction[1]);
                    }
                }
                if !found_player {
                    warn!(
                        "[WAYPOINT-TRACE] host_apply_input: NO ENTITY found for player_id={:?}",
                        player_id
                    );
                }
            }
            NetworkMessage::InteractionRequest {
                player_id: _,
                position,
                interaction_type,
            } => {
                let rel_x = position[0] as i64;
                let rel_y = position[1] as i64;
                let rel_z = position[2] as i64;

                if rel_x >= 0
                    && rel_y >= 0
                    && rel_z >= 0
                    && rel_x < params.board_field.0.shape()[0] as i64
                    && rel_y < params.board_field.0.shape()[1] as i64
                    && rel_z < params.board_field.0.shape()[2] as i64
                {
                    let entities =
                        &params.board_field.0[[rel_x as usize, rel_y as usize, rel_z as usize]];
                    for &entity in entities {
                        params.ev_interaction.write(ExecuteInteractionEvent {
                            entity,
                            ietype: interaction_type.clone(),
                            force_tuid: None,
                        });
                    }
                }
            }
            NetworkMessage::RequestTruckEntry { player_id } => {
                debug!(
                    "Network: Received RequestTruckEntry from client {:?}",
                    player_id
                );
                for (entity, id, p_pos, _) in params.query_player_pos.iter() {
                    if id == player_id {
                        let mut near_van = false;
                        for (v_pos, v_beh) in params.query_van.iter() {
                            if v_beh.is_van_entry() && p_pos.distance(v_pos) < 2.0 {
                                near_van = true;
                                break;
                            }
                        }

                        if near_van {
                            info!(
                                "Network: RequestTruckEntry validated for player {:?}. Adding InTruck.",
                                player_id
                            );
                            params
                                .commands
                                .entity(entity)
                                .insert(InTruck)
                                .insert(Hiding { hiding_spot: None });
                        } else {
                            warn!(
                                "Network: Rejected RequestTruckEntry for player {:?} - too far from van",
                                player_id
                            );
                        }
                        break;
                    }
                }
            }
            NetworkMessage::RequestTruckExit { player_id } => {
                debug!(
                    "Network: Received RequestTruckExit from client {:?}",
                    player_id
                );
                for (entity, id, _, _) in params.query_player_pos.iter() {
                    if id == player_id {
                        params
                            .commands
                            .entity(entity)
                            .remove::<InTruck>()
                            .remove::<Hiding>();
                        break;
                    }
                }
            }
            NetworkMessage::RequestEndMission => {
                debug!("Network: Received RequestEndMission from client. Validating.");
                if params.mission_end_requested.0 {
                    info!("Network: RequestEndMission validated. Ending mission.");
                    params
                        .ev_mission
                        .write(unevents_core::events::mission::MissionEvent::End);
                } else {
                    warn!("Network: RequestEndMission received but conditions not met.");
                }
            }
            NetworkMessage::CraftRepellent {
                player_id,
                ghost_type,
            } => {
                debug!(
                    "Network: Received CraftRepellent from client {:?} for {:?}",
                    player_id, ghost_type
                );
                // 1. Validate player is in truck
                let mut p_data = None;
                for (entity, id, _, is_in_truck) in params.query_player_pos.iter() {
                    if id == player_id {
                        p_data = Some((entity, is_in_truck));
                        break;
                    }
                }

                if let Some((_entity, is_in_truck)) = p_data {
                    if is_in_truck {
                        // 2. Validate craft limits
                        if params.craft_tracker.can_craft() {
                            // 3. Perform craft
                            // Find the gear for this player
                            for (_e, id, _, mut gear) in params.query_players.iter_mut() {
                                if id == player_id {
                                    let consumed_new_bottle =
                                        untruck_plugin::craft_repellent::craft_repellent(
                                            &mut params.commands,
                                            &params.gear_registry,
                                            &mut gear,
                                            *ghost_type,
                                            &mut params.q_repellent,
                                            &params.q_gearkind,
                                        );

                                    // Ensure any newly spawned gear has a NetworkId
                                    let mut ensure_net_id = |entity: Entity| {
                                        if params.query_net_id.get(entity).is_err() {
                                            let mut rng = rand::rng();
                                            let net_id =
                                                NetworkId(rng.random_range(1000..u64::MAX));
                                            params.commands.entity(entity).insert(net_id);
                                            debug!(
                                                "Network: Assigned {:?} to local entity {:?}",
                                                net_id, entity
                                            );
                                        }
                                    };
                                    if let Some(e) = gear.left_hand {
                                        ensure_net_id(e);
                                    }
                                    if let Some(e) = gear.right_hand {
                                        ensure_net_id(e);
                                    }

                                    if consumed_new_bottle {
                                        params.craft_tracker.craft();
                                        info!(
                                            "Network: Crafted repellent for remote player {:?}",
                                            player_id
                                        );

                                        // Play sound at player position
                                        params
                                            .commands
                                            .spawn(AudioPlayer::new(
                                                params
                                                    .asset_server
                                                    .load("sounds/effects-dingdingding.ogg"),
                                            ))
                                            .insert(PlaybackSettings {
                                                mode: bevy::audio::PlaybackMode::Despawn,
                                                volume: bevy::audio::Volume::Linear(
                                                    1.0 * params
                                                        .audio_settings
                                                        .volume_master
                                                        .as_f32()
                                                        * params
                                                            .audio_settings
                                                            .volume_effects
                                                            .as_f32(),
                                                ),
                                                ..Default::default()
                                            });
                                    }

                                    // Client should close UI themselves upon receiving the update,
                                    // or we could send a command.
                                    // For now, removing InTruck will force them out if they sync state.
                                    // But they stay in Truck state locally.
                                    break;
                                }
                            }
                        } else {
                            warn!(
                                "Network: CraftRepellent rejected for {:?} - limit reached",
                                player_id
                            );
                        }
                    } else {
                        warn!(
                            "Network: CraftRepellent rejected for {:?} - not in truck",
                            player_id
                        );
                    }
                }
            }
            NetworkMessage::RequestTruckInventoryChange { player_id, change } => {
                for (_entity, id, _input, mut p_gear) in params.query_players.iter_mut() {
                    if id == player_id {
                        match change {
                            unnet_core::messages::TruckInventoryChange::RemoveLeftHand => {
                                if let Some(e) = p_gear.left_hand.take() {
                                    params.commands.entity(e).despawn();
                                }
                            }
                            unnet_core::messages::TruckInventoryChange::RemoveRightHand => {
                                if let Some(e) = p_gear.right_hand.take() {
                                    params.commands.entity(e).despawn();
                                }
                            }
                            unnet_core::messages::TruckInventoryChange::RemoveInventoryIndex(
                                idx,
                            ) => {
                                if *idx < p_gear.inventory.len() {
                                    let e = p_gear.inventory.remove(*idx);
                                    params.commands.entity(e).despawn();
                                }
                            }
                            unnet_core::messages::TruckInventoryChange::AddItem(kind) => {
                                if *kind != ungear_core::types::gear::kind::GearKind::None {
                                    let entity =
                                        params.gear_registry.spawn(&mut params.commands, *kind);
                                    let mut rng = rand::rng();
                                    let net_id = NetworkId(rng.random_range(1000..u64::MAX));
                                    params.commands.entity(entity).insert(net_id);

                                    if p_gear.left_hand.is_none() {
                                        p_gear.left_hand = Some(entity);
                                    } else if p_gear.right_hand.is_none() {
                                        p_gear.right_hand = Some(entity);
                                    } else if p_gear.inventory.len() < 2 {
                                        p_gear.inventory.push(entity);
                                    } else {
                                        params.commands.entity(entity).despawn();
                                    }
                                }
                            }
                        }
                        // Issue 2 Fix: Force full sync when inventory changes relative to the truck
                        // This ensures clients process the despawned entities correctly
                        if let Some(conn) = &mut params.network_conn
                            && let NetworkConn::Active {
                                needs_full_sync, ..
                            } = &mut **conn
                        {
                            *needs_full_sync = true;
                        }
                        break;
                    }
                }
            }
            NetworkMessage::GrabRequest(msg) => {
                for (_, id, mut input, _) in params.query_players.iter_mut() {
                    if id == &msg.player_id {
                        input.grab = true;
                    }
                }
            }
            NetworkMessage::DropRequest { player_id } => {
                for (_, id, mut input, _) in params.query_players.iter_mut() {
                    if id == player_id {
                        input.drop = true;
                    }
                }
            }
            NetworkMessage::CycleInventoryRequest { player_id } => {
                for (_, id, mut input, _) in params.query_players.iter_mut() {
                    if id == player_id {
                        input.inventory_cycle = true;
                    }
                }
            }
            NetworkMessage::SwapHandsRequest { player_id } => {
                for (_, id, mut input, _) in params.query_players.iter_mut() {
                    if id == player_id {
                        input.inventory_swap = true;
                    }
                }
            }
            NetworkMessage::PlayerLeft { player_id } => {
                info!("Network: Received PlayerLeft from client {:?}", player_id);
                for (entity, id, _, _) in params.query_player_pos.iter() {
                    if id == player_id {
                        info!("Network: Despawning player entity for {:?}", id);
                        params.commands.entity(entity).despawn();
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    measure.end_ms();
}

pub(crate) fn host_send_summary_system(
    mut conn: ResMut<NetworkConn>,
    cli: Res<CliOptions>,
    summary_data: Res<SummaryData>,
) {
    let measure = metrics::HOST_SEND_SUMMARY.time_measure();
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        measure.end_ms();
        return;
    }
    info!("Network: Sending MissionSummary to clients");
    conn.send(NetworkMessage::MissionSummary {
        result: unnet_core::messages::MissionResult {
            time_taken_secs: summary_data.time_taken_secs,
            ghost_types: summary_data.ghost_types.clone(),
            repellent_used_amt: summary_data.repellent_used_amt,
            ghosts_unhaunted: summary_data.ghosts_unhaunted,
            base_score: summary_data.base_score,
            difficulty_multiplier: summary_data.difficulty_multiplier,
            grade_multiplier: summary_data.grade_multiplier,
            average_sanity: summary_data.average_sanity,
            player_count: summary_data.player_count as u32,
            alive_count: summary_data.alive_count as u32,
            full_score: summary_data.full_score,
            mission_successful: summary_data.mission_successful,
            money_earned: summary_data.money_earned,
            grade_achieved: summary_data.grade_achieved,
            required_deposit: summary_data.required_deposit,
            mission_reward_base: summary_data.mission_reward_base,
            deposit_originally_held: summary_data.deposit_originally_held,
            deposit_returned_to_bank: summary_data.deposit_returned_to_bank,
            costs_deducted_from_deposit: summary_data.costs_deducted_from_deposit,
        },
    });
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

pub(crate) fn client_process_pending_map(
    mut pending_map: ResMut<crate::resources::PendingMapLoad>,
    maps: Res<Maps>,
    mut ev_load_level: MessageWriter<LoadLevelEvent>,
) {
    let measure = metrics::CLIENT_PROCESS_PENDING_MAP.time_measure();
    // Clone path to avoid holding borrow on pending_map
    let Some(path) = pending_map.map_filepath.clone() else {
        measure.end_ms();
        return;
    };

    // Check if that path exists in maps.maps
    if maps.maps.iter().any(|m| m.path == path) {
        info!("Network: Map assets ready. Loading map: {}", path);
        ev_load_level.write(LoadLevelEvent {
            map_filepath: path.clone(),
        });
        pending_map.map_filepath = None;
    } else {
        trace!("Network: Waiting for map asset to be ready: {}", path);
    }
    measure.end_ms();
}

pub(crate) fn client_request_grab_system(
    mut conn: ResMut<NetworkConn>,
    cli: Res<CliOptions>,
    local_id: Res<LocalPlayer>,
    query_player: Query<&PlayerInput, With<MainPlayer>>,
) {
    let measure = metrics::CLIENT_REQUEST_GRAB.time_measure();
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        measure.end_ms();
        return;
    }
    if !conn.is_active() {
        measure.end_ms();
        return;
    }

    let Some(player_id) = local_id.0 else {
        measure.end_ms();
        return;
    };

    for input in query_player.iter() {
        if input.grab {
            conn.send(NetworkMessage::GrabRequest(
                unnet_core::messages::GrabRequestMsg {
                    player_id,
                    target_id: NetworkId::default(),
                },
            ));
        }
        if input.drop {
            conn.send(NetworkMessage::DropRequest { player_id });
        }
        if input.inventory_cycle {
            conn.send(NetworkMessage::CycleInventoryRequest { player_id });
        }
        if input.inventory_swap {
            conn.send(NetworkMessage::SwapHandsRequest { player_id });
        }
    }
    measure.end_ms();
}
