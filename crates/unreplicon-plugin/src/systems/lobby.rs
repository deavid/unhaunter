use bevy::prelude::*;
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientId, ClientMessageAppExt, ConnectedClient, FromClient, Replicated,
    ServerState,
};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty::Difficulty;
use unmapload_core::events::loadlevel::LoadLevelEvent;
use unprofile_core::profile::RuntimeInstallationId;
use unreplicon_core::components::{LobbyInfo, LobbyPlayerInfo, SelectedMission, ServerGamePhase};
use unreplicon_core::messages::{
    RequestAbortMission, RequestSelectDifficulty, RequestSelectMap, RequestStartMission,
};
use unreplicon_core::ownership::{Owner, OwnerId};
use unreplicon_core::resources::{
    ClientUuidMap, CurrentMapSeed, HostGone, LocalPlayer, MissionAutoJoinArmed,
};
use untypes_core::roles::{AuthorityRole, LocalPlayerRole};
use untypes_core::states::{AppState, BootState, GameState, SimulationState};
use uuid::Uuid;

const AUTO_JOIN_BUFFER_SECS: f32 = 2.0;
const AUTO_JOIN_WAITING_LOBBY_SECS: f32 = 0.75;
const AUTO_JOIN_MAX_MISSION_AGE_SECS: f64 = 30.0;

pub(super) fn app_setup(app: &mut App) {
    // Register client → server messages
    app.add_client_message::<RequestSelectMap>(Channel::Ordered);
    app.add_client_message::<RequestSelectDifficulty>(Channel::Ordered);
    app.add_client_message::<RequestStartMission>(Channel::Ordered);
    app.add_client_message::<RequestAbortMission>(Channel::Ordered);

    // Register replicated components
    app.replicate::<LobbyInfo>();
    app.replicate::<ServerGamePhase>();
    app.replicate::<SelectedMission>();

    // Register local UI messages
    app.add_message::<untypes_core::roles::DisconnectRequest>();

    // Initialize resources that are referenced by lobby UI systems.
    app.init_resource::<ClientUuidMap>();
    app.init_resource::<LocalPlayer>();
    app.init_resource::<CurrentMapSeed>();
    app.init_resource::<MissionAutoJoinArmed>();
    app.init_resource::<HostGone>();
    app.init_resource::<unreplicon_core::resources::RoomIdentification>();

    // Observer: fires whenever a client entity loses ConnectedClient on disconnect.
    app.add_observer(on_client_disconnected);

    app.add_systems(
        Update,
        process_newly_connected_clients.run_if(resource_exists::<AuthorityRole>),
    );
    app.add_systems(
        Update,
        reconcile_reconnected_player_ownership
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(SimulationState::Ready)),
    );

    // Two hooks so the transition to Lobby is caught regardless of which state
    // settles last.  On a dedicated server, ServerState::Running fires at startup
    // (before maps load) and BootState::Ready fires once maps are ready — both
    // can be the "later" one depending on timing.
    app.add_systems(
        Update,
        auto_start_headless_lobby.run_if(
            (in_state(ServerState::Running).or(resource_exists::<AuthorityRole>))
                .and(in_state(BootState::Ready))
                .and(in_state(AppState::MainMenu)),
        ),
    );

    // Server-side lobby lifecycle
    app.add_systems(
        Update,
        spawn_lobby_entity_if_missing.run_if(resource_exists::<AuthorityRole>),
    );
    app.add_systems(
        OnEnter(AppState::Lobby),
        reset_lobby_entity_on_reenter.run_if(resource_exists::<AuthorityRole>),
    );

    // Server-side: broadcast InGame state to clients when the mission starts.
    app.add_systems(
        OnEnter(untypes_core::states::SimulationState::Ready),
        set_server_state_ingame.run_if(resource_exists::<AuthorityRole>),
    );

    // Server-side message handlers
    app.add_systems(
        Update,
        (
            handle_request_select_map,
            handle_request_select_difficulty,
            handle_request_start_mission,
            handle_request_abort_mission,
        )
            .run_if(resource_exists::<AuthorityRole>),
    );

    // Observe SimulationState::Ready to transition MissionLoading → InGame
    app.add_systems(
        Update,
        observe_simulation_ready_to_enter_game.run_if(in_state(AppState::MissionLoading)),
    );

    app.add_systems(Update, process_auto_join);
}

fn process_auto_join(
    q_mission: Query<&SelectedMission>,
    q_server_phase: Query<&ServerGamePhase>,
    local_player: Option<Res<LocalPlayerRole>>,
    authority: Option<Res<AuthorityRole>>,
    app_state: Res<State<AppState>>,
    sim_state: Res<State<SimulationState>>,
    mut current_map_seed: ResMut<CurrentMapSeed>,
    mut current_difficulty: ResMut<CurrentDifficulty>,
    mut ev_load: MessageWriter<LoadLevelEvent>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut auto_join_armed: ResMut<MissionAutoJoinArmed>,
    time: Res<Time>,
    mut sync_timer: Local<Option<f32>>,
    mut lobby_wait_started_at: Local<Option<f32>>,
    mut tracked_mission_seed: Local<Option<u64>>,
    mut tracked_mission_joinable: Local<bool>,
) {
    if authority.is_some() || local_player.is_none() {
        auto_join_armed.0 = false;
        return;
    }
    if *app_state.get() != AppState::Lobby {
        auto_join_armed.0 = false;
        *sync_timer = None;
        *lobby_wait_started_at = None;
        *tracked_mission_seed = None;
        *tracked_mission_joinable = false;
        return;
    }

    let now = time.elapsed_secs();
    if lobby_wait_started_at.is_none() {
        *lobby_wait_started_at = Some(now);
    }

    let Some(mission) = q_mission.iter().next() else {
        auto_join_armed.0 = false;
        *sync_timer = None;
        *tracked_mission_seed = None;
        *tracked_mission_joinable = false;
        return;
    };

    if *tracked_mission_seed != Some(mission.map_seed) {
        *tracked_mission_seed = Some(mission.map_seed);
        *sync_timer = None;

        let waited_in_lobby_long_enough = lobby_wait_started_at
            .map(|started| now - started >= AUTO_JOIN_WAITING_LOBBY_SECS)
            .unwrap_or(false);

        let mission_age_ok = current_unix_time_secs()
            .map(|unix_now| {
                mission.started_at_unix_secs > 0.0
                    && unix_now >= mission.started_at_unix_secs
                    && (unix_now - mission.started_at_unix_secs) <= AUTO_JOIN_MAX_MISSION_AGE_SECS
            })
            .unwrap_or(false);

        *tracked_mission_joinable = waited_in_lobby_long_enough && mission_age_ok;
    }

    if !*tracked_mission_joinable {
        auto_join_armed.0 = false;
        return;
    }

    auto_join_armed.0 = true;

    let ready_by_sim_state = *sim_state.get() == SimulationState::Ready;
    let ready_by_server_phase = q_server_phase
        .iter()
        .any(|phase| *phase == ServerGamePhase::InProgress);
    if !ready_by_sim_state && !ready_by_server_phase {
        *sync_timer = None;
        return;
    }

    let start = sync_timer.get_or_insert(now);
    if now - *start < AUTO_JOIN_BUFFER_SECS {
        return;
    }

    current_map_seed.0 = mission.map_seed;
    if let Ok(diff) = Difficulty::from_str(&mission.difficulty_id) {
        *current_difficulty = CurrentDifficulty::new(diff);
    } else {
        warn!(
            "process_auto_join: unknown difficulty '{}'; keeping current",
            mission.difficulty_id
        );
    }
    ev_load.write(LoadLevelEvent {
        map_filepath: mission.map_path.clone(),
    });
    next_app_state.set(AppState::MissionLoading);
    auto_join_armed.0 = false;
    *sync_timer = None;
}

fn current_unix_time_secs() -> Option<f64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs_f64())
}

/// Helper to get UUID for a Replicon ClientId
fn client_uuid(client_id: ClientId, uuid_map: &Res<ClientUuidMap>) -> Option<Uuid> {
    let owner_id = match client_id {
        ClientId::Server => OwnerId::Server,
        ClientId::Client(e) => OwnerId::Client(e),
    };
    uuid_map.0.get(&owner_id).copied()
}

/// Helper: convert Replicon ClientId to OwnerId.
fn to_owner_id(client_id: ClientId) -> OwnerId {
    match client_id {
        ClientId::Server => OwnerId::Server,
        ClientId::Client(e) => OwnerId::Client(e),
    }
}

/// Server: In hub-less dedicated mode, transition to Lobby immediately.
fn auto_start_headless_lobby(
    procman: Option<Res<crate::systems::procman::ProcManChannel>>,
    mut next_state: ResMut<NextState<AppState>>,
    authority: Option<Res<AuthorityRole>>,
    local_player: Option<Res<LocalPlayerRole>>,
    cli: Res<untypes_core::cli::CliOptions>,
) {
    let is_dedicated = cli.dedicated;
    let is_authority = authority.is_some();
    let is_local_player = local_player.is_some();
    let has_procman = procman.is_some();

    // Dedicated server (authority, NO local player) or hub-less direct-connect authority.
    if is_dedicated || (is_authority && !is_local_player && !has_procman) {
        info!(
            "Dedicated mode detected (dedicated={is_dedicated}, auth={is_authority}, local={is_local_player}, procman={has_procman}). Auto-transitioning to AppState::Lobby"
        );
        // We set both states for better compatibility, although dedicated servers
        // usually only care about AppState.
        next_state.set(AppState::Lobby);
    }
}

/// Spawn the authoritative lobby-state entity if it's missing.
fn spawn_lobby_entity_if_missing(
    q_lobby: Query<(), With<LobbyInfo>>,
    mut commands: Commands,
    local_player: Option<Res<LocalPlayerRole>>,
    runtime_id: Option<Res<RuntimeInstallationId>>,
    mut uuid_map: ResMut<ClientUuidMap>,
) {
    if !q_lobby.is_empty() {
        return;
    }

    // First-time spawn.
    let mut players = Vec::new();
    let mut leader_uuid = None;

    if local_player.is_some()
        && let Some(id) = runtime_id
    {
        let uuid = id.0;
        players.push(LobbyPlayerInfo {
            player_uuid: uuid,
            current_socket: None, // Local host player
            tint_color_index: 0,
            connected: true,
            nickname: None,
        });
        leader_uuid = Some(uuid);
        uuid_map.0.insert(OwnerId::Server, uuid);
    }

    commands.spawn((
        Replicated,
        LobbyInfo {
            players,
            selected_map: None,
            selected_difficulty: "standard-challenge".to_string(),
            leader_uuid,
        },
        ServerGamePhase::Lobby,
    ));
    info!("Lobby entity spawned (leader={:?})", leader_uuid);
}

/// Reset the authoritative lobby-state entity when the server re-enters Lobby.
fn reset_lobby_entity_on_reenter(
    mut q_existing: Query<(&mut LobbyInfo, &mut ServerGamePhase)>,
    q_selected_mission: Query<Entity, With<SelectedMission>>,
    mut commands: Commands,
) {
    if let Ok((mut lobby, mut game_phase)) = q_existing.single_mut() {
        // Re-entering Lobby after a mission: reset selection, signal state change.
        *game_phase = ServerGamePhase::Lobby;
        lobby.selected_map = None;
        lobby.set_changed();
        info!("Lobby entity reset for new session (phase set to Lobby)");
    } else {
        warn!("reset_lobby_entity_on_reenter: Lobby entity not found!");
    }

    for entity in q_selected_mission.iter() {
        commands.entity(entity).despawn();
        info!("SelectedMission entity despawned");
    }
}

/// Server: write `ServerGamePhase::InProgress` on the lobby entity when the server
/// enters `AppState::InGame`.
fn set_server_state_ingame(mut q: Query<(&mut ServerGamePhase, &mut LobbyInfo)>) {
    for (mut phase, mut lobby) in q.iter_mut() {
        *phase = ServerGamePhase::InProgress;
        lobby.set_changed();
    }
    info!("ServerGamePhase set to InProgress");
}

/// Observe `SimulationState::Ready` and transition `AppState::MissionLoading → AppState::InGame`.
fn observe_simulation_ready_to_enter_game(
    sim_state: Res<State<untypes_core::states::SimulationState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut frame: Local<u32>,
) {
    if *sim_state == untypes_core::states::SimulationState::Ready {
        info!("Simulation ready; transitioning MissionLoading -> InGame");
        next_app_state.set(AppState::InGame);
    } else {
        *frame += 1;
        if frame.is_multiple_of(120) {
            debug!("Waiting for simulation to be ready - current state: {sim_state:?}");
        }
    }
}

/// System: triggered every frame on the server to handle clients that have connected
/// but haven't been added to the lobby yet (e.g. waiting for authentication).
fn process_newly_connected_clients(
    q_clients: Query<Entity, With<ConnectedClient>>,
    mut q_lobby: Query<&mut LobbyInfo>,
    uuid_map: Res<ClientUuidMap>,
) {
    let client_count = q_clients.iter().count();
    if q_lobby.is_empty() {
        // Log only when clients are present so we notice if the lobby entity never spawns.
        if client_count > 0 {
            warn!(
                "process_newly_connected_clients: {} client(s) waiting but lobby entity not yet spawned!",
                client_count
            );
        }
        return;
    }

    for entity in q_clients.iter() {
        let client_id = ClientId::Client(entity);
        let Some(uuid) = client_uuid(client_id, &uuid_map) else {
            debug!(
                "Still waiting for authentication/mapping: (socket={:?})",
                client_id
            );

            continue;
        };

        for mut lobby in q_lobby.iter_mut() {
            let owner_id = to_owner_id(client_id);
            // Check if player is already in the list
            if let Some(player) = lobby.players.iter_mut().find(|p| p.player_uuid == uuid) {
                if player.current_socket != Some(owner_id) || !player.connected {
                    player.current_socket = Some(owner_id);
                    player.connected = true;
                    // Trigger replication by re-setting the field (even if same value,
                    // though if we reached here something changed).
                    lobby.set_changed();
                    info!("Player {} reconnected (socket={:?})", uuid, client_id);
                }
            } else {
                // New player
                // Find the lowest colour slot (0..=8) not currently used by any player.
                let mut used = [false; 9];
                for p in lobby.players.iter() {
                    let idx = p.tint_color_index as usize;
                    if idx < 9 {
                        used[idx] = true;
                    }
                }
                let color_index = used.iter().position(|&u| !u).unwrap_or(9) as u8;

                lobby.players.push(LobbyPlayerInfo {
                    player_uuid: uuid,
                    current_socket: Some(owner_id),
                    tint_color_index: color_index,
                    connected: true,
                    nickname: None,
                });
                info!(
                    "Player {} joined lobby (socket={:?}, tint={})",
                    uuid, client_id, color_index
                );
            }

            // If lobby has no leader, assign the first player (new or reconnected).
            if lobby.leader_uuid.is_none() {
                lobby.leader_uuid = Some(uuid);
                info!("Player {} assigned as lobby leader", uuid);
            }
        }
    }
}

/// Server: while a mission is running, reconcile ownership of already-spawned player and gear
/// entities after a reconnect.
///
/// Reconnects can change the underlying socket owner from `Client(...v0)` to `Client(...v1)`.
/// If avatar/gear entities keep the old `Owner`, state exports from the reconnected client are
/// rejected as sender mismatches.
fn reconcile_reconnected_player_ownership(
    q_lobby: Query<&LobbyInfo>,
    q_players: Query<(
        Entity,
        &unplayer_core::components::PlayerSprite,
        &ungear_core::components::playergear::PlayerGear,
        &Owner,
        Has<unplayer_core::components::PlayerDisconnected>,
    )>,
    q_owned: Query<&Owner>,
    mut commands: Commands,
) {
    let Ok(lobby) = q_lobby.single() else {
        return;
    };

    for lobby_player in &lobby.players {
        if !lobby_player.connected {
            continue;
        }

        let Some(expected_owner) = lobby_player.current_socket else {
            continue;
        };

        let Some((player_entity, _sprite, player_gear, owner, is_disconnected)) = q_players
            .iter()
            .find(|(_entity, sprite, _gear, _owner, _disconnected)| {
                sprite.id == lobby_player.player_uuid
            })
        else {
            continue;
        };

        let mut changed = false;

        if owner.0 != expected_owner {
            commands.entity(player_entity).insert(Owner(expected_owner));
            changed = true;
        }

        for gear_entity in player_gear
            .left_hand
            .into_iter()
            .chain(player_gear.right_hand)
            .chain(player_gear.inventory.iter().copied())
        {
            let Ok(gear_owner) = q_owned.get(gear_entity) else {
                continue;
            };

            if gear_owner.0 != expected_owner {
                commands.entity(gear_entity).insert(Owner(expected_owner));
                changed = true;
            }
        }

        if is_disconnected {
            commands
                .entity(player_entity)
                .remove::<unplayer_core::components::PlayerDisconnected>();
            changed = true;
        }

        if changed {
            info!(
                "Reconciled ownership after reconnect for player {} (owner={:?})",
                lobby_player.player_uuid, expected_owner
            );
        }
    }
}

fn on_client_disconnected(
    trigger: On<Remove, ConnectedClient>,
    mut q_lobby: Query<(&mut LobbyInfo, &ServerGamePhase)>,
    uuid_map: Res<ClientUuidMap>,
    mut commands: Commands,
    q_sprites: Query<(Entity, &unplayer_core::components::PlayerSprite)>,
) {
    let client_id = ClientId::Client(trigger.entity);
    let Some(uuid) = client_uuid(client_id, &uuid_map) else {
        return;
    };

    for (mut lobby, phase) in q_lobby.iter_mut() {
        if *phase == ServerGamePhase::Lobby {
            // Hard remove
            lobby.players.retain(|p| p.player_uuid != uuid);
            info!(
                "Player {} hard-removed from lobby (ServerGamePhase::Lobby)",
                uuid
            );
        } else {
            // Soft remove
            if let Some(player) = lobby.players.iter_mut().find(|p| p.player_uuid == uuid) {
                player.connected = false;
                player.current_socket = None;
                info!("Player {} soft-disconnected (phase={:?})", uuid, phase);
            }
            if let Some((entity, _)) = q_sprites.iter().find(|(_, s)| s.id == uuid) {
                commands
                    .entity(entity)
                    .insert(unplayer_core::components::PlayerDisconnected);
                info!(
                    "Inserted PlayerDisconnected on avatar entity for player {}",
                    uuid
                );
            }
        }

        // If the leader left, assign leadership to the next connected player.
        if lobby.leader_uuid == Some(uuid) {
            lobby.leader_uuid = lobby
                .players
                .iter()
                .find(|p| p.connected && p.current_socket.is_some())
                .map(|p| p.player_uuid);
            info!(
                "Leader disconnected; new leader_uuid={:?}",
                lobby.leader_uuid
            );
        }
    }
}

fn handle_request_select_map(
    mut reader: MessageReader<FromClient<RequestSelectMap>>,
    mut q_lobby: Query<&mut LobbyInfo>,
    uuid_map: Res<ClientUuidMap>,
) {
    for msg in reader.read() {
        let Some(sender_uuid) = client_uuid(msg.client_id, &uuid_map) else {
            continue;
        };
        for mut lobby in q_lobby.iter_mut() {
            if Some(sender_uuid) != lobby.leader_uuid {
                warn!(
                    "RequestSelectMap from non-leader {:?}; ignored",
                    sender_uuid
                );
                continue;
            }
            info!("Map selected: {}", msg.message.map_filepath);
            lobby.selected_map = Some(msg.message.map_filepath.clone());
        }
    }
}

fn handle_request_select_difficulty(
    mut reader: MessageReader<FromClient<RequestSelectDifficulty>>,
    mut q_lobby: Query<&mut LobbyInfo>,
    uuid_map: Res<ClientUuidMap>,
) {
    for msg in reader.read() {
        trace!(
            "Server received difficulty request from ClientId: {:?}",
            msg.client_id
        );

        let Some(sender_uuid) = client_uuid(msg.client_id, &uuid_map) else {
            warn!("Could not find UUID for ClientId: {:?}", msg.client_id);
            continue;
        };
        for mut lobby in q_lobby.iter_mut() {
            if Some(sender_uuid) != lobby.leader_uuid {
                warn!(
                    "RequestSelectDifficulty from non-leader {:?}; ignored",
                    sender_uuid
                );
                continue;
            }
            info!("Difficulty selected: {}", msg.message.difficulty_id);
            lobby.selected_difficulty = msg.message.difficulty_id.clone();
        }
    }
}

fn handle_request_start_mission(
    mut reader: MessageReader<FromClient<RequestStartMission>>,
    q_lobby: Query<&LobbyInfo>,
    uuid_map: Res<ClientUuidMap>,
    mut ev_load: MessageWriter<LoadLevelEvent>,
    mut commands: Commands,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    for msg in reader.read() {
        let Some(sender_uuid) = client_uuid(msg.client_id, &uuid_map) else {
            continue;
        };
        for lobby in q_lobby.iter() {
            if Some(sender_uuid) != lobby.leader_uuid {
                warn!(
                    "RequestStartMission from non-leader {:?}; ignored",
                    sender_uuid
                );
                continue;
            }
            let Some(map_filepath) = &lobby.selected_map else {
                warn!("RequestStartMission but no map selected; ignored");
                continue;
            };
            info!("Starting mission: {}", map_filepath);
            ev_load.write(LoadLevelEvent {
                map_filepath: map_filepath.clone(),
            });
            next_app_state.set(AppState::MissionLoading);
            // Replicate mission info to connected clients so they can join.
            commands.spawn((
                Replicated,
                SelectedMission {
                    map_path: map_filepath.clone(),
                    map_seed: msg.message.map_seed,
                    difficulty_id: lobby.selected_difficulty.clone(),
                    started_at_unix_secs: current_unix_time_secs().unwrap_or(0.0),
                },
            ));
        }
    }
}

fn handle_request_abort_mission(
    mut reader: MessageReader<FromClient<RequestAbortMission>>,
    q_lobby: Query<&LobbyInfo>,
    uuid_map: Res<ClientUuidMap>,
    q_selected_mission: Query<Entity, With<SelectedMission>>,
    mut q_server_phase: Query<&mut ServerGamePhase>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let Some(sender_uuid) = client_uuid(msg.client_id, &uuid_map) else {
            continue;
        };
        for lobby in q_lobby.iter() {
            if Some(sender_uuid) != lobby.leader_uuid {
                warn!(
                    "RequestAbortMission from non-leader {:?}; ignored",
                    sender_uuid
                );
                continue;
            }
            info!("Aborting mission");
            for entity in q_selected_mission.iter() {
                commands.entity(entity).despawn();
            }
            // Trigger the same teardown path as MissionEvent::End so that
            // cleanup_mission_players despawns existing PlayerSprite entities
            // and server_teardown_grace_period eventually returns to AppState::Lobby.
            next_game_state.set(GameState::Running);
            next_sim_state.set(SimulationState::TearingDown);
            for mut phase in q_server_phase.iter_mut() {
                *phase = ServerGamePhase::Concluding;
            }
        }
    }
}
