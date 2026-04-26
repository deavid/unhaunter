use bevy::prelude::*;
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientId, ClientMessageAppExt, ConnectedClient, FromClient, Replicated,
    ServerState,
};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use uncommon_states_core::{BootState, UIContextState};
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty::Difficulty;
use unmapload_core::events::loadlevel::LoadLevelEvent;
use unmission_core::resources::MissionEndRequested;
use unmission_core::types::MissionEvent;
use unmission_core::types::SimulationState;
use unreplicon_core::components::{LobbyInfo, LobbyPlayerInfo, SelectedMission, ServerGamePhase};
use unreplicon_core::events::{PlayerNetworkDisconnected, PlayerNetworkReconnected};
use unreplicon_core::messages::{
    MissionEndReason, RequestEndMission, RequestJoinMission, RequestSelectDifficulty,
    RequestSelectMap, RequestStartMission,
};
use unreplicon_core::ownership::{Owner, OwnerId};
use unreplicon_core::resources::LobbyPresenceRole;
use unreplicon_core::resources::{AuthorityRole, DisconnectRequest, LocalPlayerRole};
use unreplicon_core::resources::{ClientUuidMap, CurrentMapSeed, HostGone, LocalPlayer};
use untmxmap_core::resources::maps::Maps;
use uuid::Uuid;

pub(super) fn app_setup(app: &mut App) {
    // Register client → server messages
    app.add_client_message::<RequestSelectMap>(Channel::Ordered);
    app.add_client_message::<RequestSelectDifficulty>(Channel::Ordered);
    app.add_client_message::<RequestStartMission>(Channel::Ordered);
    app.add_client_message::<RequestJoinMission>(Channel::Ordered);
    app.add_client_message::<RequestEndMission>(Channel::Ordered);

    // Register replicated components
    app.replicate::<LobbyInfo>();
    app.replicate::<ServerGamePhase>();
    app.replicate::<SelectedMission>();

    // Register local UI messages
    app.add_message::<DisconnectRequest>();
    app.add_message::<PlayerNetworkDisconnected>();
    app.add_message::<PlayerNetworkReconnected>();

    // Initialize resources that are referenced by lobby UI systems.
    app.init_resource::<ClientUuidMap>();
    app.init_resource::<LocalPlayer>();
    app.init_resource::<CurrentMapSeed>();
    app.init_resource::<unreplicon_core::resources::MissionAutoJoinArmed>();
    app.init_resource::<unreplicon_core::resources::MissionAutoJoinDelay>();
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

    // Dedicated server: when maps finish loading (BootState::Ready), transition
    // EngineBoot -> MainMenu so auto_start_headless_lobby can fire.
    // Non-dedicated clients get this transition from UnhaunterMapLoadPlugin's
    // bevy_asset_loader LoadingState, which is not added for dedicated servers.
    app.add_systems(OnEnter(BootState::Ready), dedicated_boot_transition);

    // Two hooks so the transition to Lobby is caught regardless of which state
    // settles last.  On a dedicated server, ServerState::Running fires at startup
    // (before maps load) and BootState::Ready fires once maps are ready — both
    // can be the "later" one depending on timing.
    app.add_systems(
        Update,
        auto_start_headless_lobby.run_if(
            (in_state(ServerState::Running).or(resource_exists::<AuthorityRole>))
                .and(in_state(BootState::Ready))
                .and(in_state(UIContextState::MainMenu)),
        ),
    );

    // Server-side lobby lifecycle.
    // Guard: AuthorityRole AND LobbyPresenceRole.
    // - Dedicated server: AuthorityRole + LobbyPresenceRole (no LocalPlayerRole) — spawns on startup.
    // - PeerHost: AuthorityRole + LocalPlayerRole + LobbyPresenceRole — spawns when in Lobby.
    // - Standalone offline player: AuthorityRole + LocalPlayerRole, NO LobbyPresenceRole — must NOT
    //   spawn a LobbyInfo; they may transition to a pure client via Hub UI and the stale entity
    //   would then coexist with the replicated one from the server, breaking q_lobby.single().
    app.add_systems(
        Update,
        spawn_lobby_entity_if_missing
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(resource_exists::<LobbyPresenceRole>),
    );
    app.add_systems(
        OnEnter(UIContextState::Lobby),
        reset_lobby_entity_on_reenter.run_if(resource_exists::<AuthorityRole>),
    );

    // Server-side: broadcast the in-progress server phase when mission simulation is ready.
    app.add_systems(
        OnEnter(SimulationState::Ready),
        set_server_state_ingame.run_if(resource_exists::<AuthorityRole>),
    );

    // Server-side message handlers
    app.add_systems(
        Update,
        (
            handle_request_select_map,
            handle_request_select_difficulty,
            handle_request_start_mission,
            handle_request_end_mission,
        )
            .run_if(resource_exists::<AuthorityRole>),
    );

    app.add_systems(
        Update,
        process_auto_join.run_if(in_state(UIContextState::Lobby)),
    );
}

fn process_auto_join(
    q_selected_mission: Query<&SelectedMission>,
    q_server_phase: Query<&ServerGamePhase>,
    mut ev_load: MessageWriter<LoadLevelEvent>,
    mut ev_join: MessageWriter<RequestJoinMission>,
    mut next_ui_state: ResMut<NextState<UIContextState>>,
    sim_state: Res<State<SimulationState>>,
    auto_join_armed: Option<ResMut<unreplicon_core::resources::MissionAutoJoinArmed>>,
    mut auto_join_delay: ResMut<unreplicon_core::resources::MissionAutoJoinDelay>,
    mut current_map_seed: ResMut<CurrentMapSeed>,
    mut current_difficulty: ResMut<CurrentDifficulty>,
    time: Res<Time>,
) {
    let Ok(mission) = q_selected_mission.single() else {
        if let Some(mut armed) = auto_join_armed {
            armed.0 = false;
        }
        auto_join_delay.0 = None;
        return;
    };

    let Some(mut armed) = auto_join_armed else {
        return;
    };

    let is_ready = *sim_state.get() == SimulationState::Ready
        || q_server_phase
            .iter()
            .any(|phase| *phase == ServerGamePhase::InProgress);

    if !armed.0 {
        if !is_ready {
            armed.0 = true;
            auto_join_delay.0 = None;
            info!(
                "process_auto_join: Armed for auto-join (map={})",
                mission.map_path
            );
        }
        return;
    }

    if is_ready {
        // Tick the 1-second delay before actually joining.
        let remaining = auto_join_delay.0.get_or_insert(1.0);
        *remaining -= time.delta_secs();
        if *remaining > 0.0 {
            return;
        }
        // Timer elapsed — fire the join.
        armed.0 = false;
        auto_join_delay.0 = None;
        current_map_seed.0 = mission.map_seed;
        if let Some(diff) = parse_difficulty_id("process_auto_join", &mission.difficulty_id) {
            *current_difficulty = CurrentDifficulty::new(diff);
        }
        info!(
            "process_auto_join: Triggering synchronized auto-join intent (map={})",
            mission.map_path
        );

        ev_join.write(RequestJoinMission);
        ev_load.write(LoadLevelEvent {
            map_filepath: mission.map_path.clone(),
        });
        next_ui_state.set(UIContextState::MissionLoading);
    }
}

fn current_unix_time_secs() -> Option<f64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs_f64())
}

fn parse_difficulty_id(context: &str, difficulty_id: &str) -> Option<Difficulty> {
    let Ok(parsed_difficulty) = Difficulty::from_str(difficulty_id) else {
        warn!(
            "{}: difficulty '{}' could not be parsed",
            context, difficulty_id
        );
        return None;
    };

    Some(parsed_difficulty)
}

fn resolve_mission_difficulty(
    map_filepath: &str,
    lobby_selected_difficulty: &str,
    maps: &Maps,
) -> Option<Difficulty> {
    // In multiplayer, the lobby difficulty selector always wins.
    // Per-map difficulty (is_campaign_mission) only applies to single-player campaign mode,
    // which is handled separately in uncampaign-plugin/unified_mission_selection.rs.
    if !maps.maps.iter().any(|map| map.path == map_filepath) {
        warn!(
            "resolve_mission_difficulty: selected map '{}' was not found in Maps resource; proceeding with lobby-selected difficulty",
            map_filepath
        );
    }

    parse_difficulty_id(
        "resolve_mission_difficulty/lobby_selected",
        lobby_selected_difficulty,
    )
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
fn dedicated_boot_transition(
    local_player: Option<Res<LocalPlayerRole>>,
    ui_state: Res<State<UIContextState>>,
    mut next_ui_state: ResMut<NextState<UIContextState>>,
) {
    if local_player.is_some() {
        return; // Only for dedicated servers (no local player)
    }
    if *ui_state.get() != UIContextState::EngineBoot {
        return;
    }
    info!("Dedicated server: BootState::Ready reached, auto-transitioning EngineBoot -> MainMenu");
    next_ui_state.set(UIContextState::MainMenu);
}

fn auto_start_headless_lobby(
    procman: Option<Res<unreplicon_transport::resources::ProcManChannel>>,
    mut next_state: ResMut<NextState<UIContextState>>,
    authority: Option<Res<AuthorityRole>>,
    local_player: Option<Res<LocalPlayerRole>>,
) {
    let is_dedicated = local_player.is_none();
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
        next_state.set(UIContextState::Lobby);
    }
}

/// Spawn the authoritative lobby-state entity if it's missing.
fn spawn_lobby_entity_if_missing(
    q_lobby: Query<(), With<LobbyInfo>>,
    mut commands: Commands,
    local_player: Option<Res<LocalPlayerRole>>,
    local_player_res: Option<Res<LocalPlayer>>,
    mut uuid_map: ResMut<ClientUuidMap>,
    current_difficulty: Res<CurrentDifficulty>,
) {
    if !q_lobby.is_empty() {
        return;
    }

    // First-time spawn.
    let mut players = Vec::new();
    let mut leader_uuid = None;

    if local_player.is_some()
        && let Some(lp) = local_player_res
        && let Some(uuid) = lp.uuid
    {
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

    let selected_difficulty = "standard-challenge".to_string();

    if let Some(lobby_difficulty) = parse_difficulty_id(
        "AUTHORITY_LOBBY_BOOTSTRAP_DIFFICULTY_STATE",
        &selected_difficulty,
    ) {
        info!(
            "AUTHORITY_LOBBY_BOOTSTRAP_DIFFICULTY_STATE: selected_difficulty={:?} authoritative_current_difficulty={:?} leader={:?}",
            lobby_difficulty, current_difficulty.0, leader_uuid
        );
    }

    commands.spawn((
        Replicated,
        LobbyInfo {
            players,
            selected_map: None,
            selected_difficulty: selected_difficulty.clone(),
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
    current_difficulty: Res<CurrentDifficulty>,
) {
    if let Ok((mut lobby, mut game_phase)) = q_existing.single_mut() {
        // Re-entering Lobby after a mission: reset selection, signal state change.
        *game_phase = ServerGamePhase::Lobby;
        lobby.selected_map = None;
        lobby.set_changed();
        if let Some(lobby_difficulty) = parse_difficulty_id(
            "AUTHORITY_LOBBY_REENTER_DIFFICULTY_STATE",
            &lobby.selected_difficulty,
        ) {
            info!(
                "AUTHORITY_LOBBY_REENTER_DIFFICULTY_STATE: selected_difficulty={:?} authoritative_current_difficulty={:?}",
                lobby_difficulty, current_difficulty.0
            );
        }
        info!("Lobby entity reset for new session (phase set to Lobby)");
    } else {
        warn!("reset_lobby_entity_on_reenter: Lobby entity not found!");
    }

    for entity in q_selected_mission.iter() {
        commands.entity(entity).despawn();
        info!("SelectedMission entity despawned");
    }
}

/// Server: write `ServerGamePhase::InProgress` on the lobby entity when mission
/// simulation becomes ready.
fn set_server_state_ingame(mut q: Query<(&mut ServerGamePhase, &mut LobbyInfo)>) {
    for (mut phase, mut lobby) in q.iter_mut() {
        *phase = ServerGamePhase::InProgress;
        lobby.set_changed();
    }
    info!("ServerGamePhase set to InProgress");
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

/// Server: while a mission is running, reconcile ownership of already-spawned player
/// entities after a reconnect.
///
/// Reconnects can change the underlying socket owner from `Client(...v0)` to `Client(...v1)`.
/// If avatar entities keep the old `Owner`, state exports from the reconnected client are
/// rejected as sender mismatches. Gear ownership is reconciled in ungear-plugin.
fn reconcile_reconnected_player_ownership(
    q_lobby: Query<&LobbyInfo>,
    q_players: Query<(
        Entity,
        &unplayer_core::components::PlayerSprite,
        &Owner,
        Has<unplayer_core::components::PlayerDisconnected>,
    )>,
    mut commands: Commands,
    mut ev_reconnected: MessageWriter<PlayerNetworkReconnected>,
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

        let Some((player_entity, _sprite, owner, is_disconnected)) = q_players
            .iter()
            .find(|(_entity, sprite, _owner, _disconnected)| sprite.id == lobby_player.player_uuid)
        else {
            continue;
        };

        let mut changed = false;

        if owner.0 != expected_owner {
            commands.entity(player_entity).insert(Owner(expected_owner));
            changed = true;
        }

        if is_disconnected || owner.0 != expected_owner {
            ev_reconnected.write(PlayerNetworkReconnected {
                player_uuid: lobby_player.player_uuid,
                new_owner_id: expected_owner,
            });
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
    mut ev_disconnected: MessageWriter<PlayerNetworkDisconnected>,
) {
    let client_id = ClientId::Client(trigger.entity);
    let Some(uuid) = client_uuid(client_id, &uuid_map) else {
        warn!(
            "on_client_disconnected: missing UUID mapping for disconnected client {:?}",
            client_id
        );
        return;
    };

    for (mut lobby, phase) in q_lobby.iter_mut() {
        let connected_before = lobby.players.iter().filter(|p| p.connected).count();
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
            ev_disconnected.write(PlayerNetworkDisconnected { player_uuid: uuid });
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

        let connected_after = lobby.players.iter().filter(|p| p.connected).count();
        debug!(
            "LOBBY_DISCONNECT_STATE: phase={:?} disconnected_uuid={} connected_before={} connected_after={} leader_uuid={:?}",
            phase, uuid, connected_before, connected_after, lobby.leader_uuid
        );
        if *phase != ServerGamePhase::Lobby && connected_after == 0 {
            warn!(
                "LOBBY_DISCONNECT_LAST_PLAYER: last connected player left during phase {:?}; mission-end fallback logic should take over",
                phase
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
    current_difficulty: Res<CurrentDifficulty>,
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
            let Some(requested_difficulty) = parse_difficulty_id(
                "AUTHORITY_LOBBY_DIFFICULTY_SELECTION_APPLIED",
                &msg.message.difficulty_id,
            ) else {
                warn!(
                    "RequestSelectDifficulty received unknown difficulty '{}' from leader {:?}; ignoring request",
                    msg.message.difficulty_id, sender_uuid
                );
                continue;
            };
            info!(
                "AUTHORITY_LOBBY_DIFFICULTY_SELECTION_APPLIED: lobby_selected_difficulty={:?} authoritative_current_difficulty={:?}",
                requested_difficulty, current_difficulty.0
            );
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
    mut next_app_state: ResMut<NextState<UIContextState>>,
    mut current_difficulty: ResMut<CurrentDifficulty>,
    maps: Res<Maps>,
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
            let previous_current_difficulty = current_difficulty.0;
            let Some(selected_difficulty) =
                resolve_mission_difficulty(map_filepath, &lobby.selected_difficulty, &maps)
            else {
                warn!(
                    "MULTIPLAYER_MISSION_START_UNKNOWN_DIFFICULTY: unable to resolve mission difficulty for map '{}' with lobby-selected difficulty '{}'",
                    map_filepath, lobby.selected_difficulty
                );
                continue;
            };
            if selected_difficulty != current_difficulty.0 {
                info!(
                    "MULTIPLAYER_MISSION_START_DIFFICULTY_SYNC: syncing authoritative CurrentDifficulty from {:?} to resolved mission difficulty {:?}",
                    current_difficulty.0, selected_difficulty
                );
                current_difficulty.0 = selected_difficulty;
            }
            info!(
                "MULTIPLAYER_MISSION_START_SERVER: map='{}' seed={} lobby_selected_difficulty='{}' resolved_mission_difficulty={:?} authoritative_current_difficulty_before={:?} authoritative_current_difficulty_after={:?}",
                map_filepath,
                msg.message.map_seed,
                lobby.selected_difficulty,
                selected_difficulty,
                previous_current_difficulty,
                current_difficulty.0
            );
            if selected_difficulty != current_difficulty.0 {
                warn!(
                    "MULTIPLAYER_MISSION_START_DIFFICULTY_MISMATCH: mission is starting with lobby difficulty {:?}, but authoritative CurrentDifficulty is {:?} even after sync. Ghost selection and other mission systems will read the resource value.",
                    selected_difficulty, current_difficulty.0
                );
            }
            info!("Starting mission: {}", map_filepath);
            ev_load.write(LoadLevelEvent {
                map_filepath: map_filepath.clone(),
            });
            next_app_state.set(UIContextState::MissionLoading);
            // Replicate mission info to connected clients so they can join.
            commands.spawn((
                Replicated,
                SelectedMission {
                    map_path: map_filepath.clone(),
                    map_seed: msg.message.map_seed,
                    difficulty_id: selected_difficulty.to_string(),
                    started_at_unix_secs: current_unix_time_secs().unwrap_or(0.0),
                },
            ));
        }
    }
}

fn handle_request_end_mission(
    mut reader: MessageReader<FromClient<RequestEndMission>>,
    q_lobby: Query<&LobbyInfo>,
    uuid_map: Res<ClientUuidMap>,
    mission_end_requested: Res<MissionEndRequested>,
    mut ev_mission: MessageWriter<MissionEvent>,
) {
    for msg in reader.read() {
        let Some(sender_uuid) = client_uuid(msg.client_id, &uuid_map) else {
            continue;
        };
        for lobby in q_lobby.iter() {
            let allowed = match msg.message.reason {
                MissionEndReason::LeaderAborted => Some(sender_uuid) == lobby.leader_uuid,
                MissionEndReason::TruckExitInitiated => mission_end_requested.0,
            };

            if !allowed {
                warn!(
                    "RequestEndMission {:?} rejected for sender {:?}",
                    msg.message.reason, sender_uuid
                );
                continue;
            }

            info!(
                "RequestEndMission {:?} accepted from sender {:?}",
                msg.message.reason, sender_uuid
            );
            ev_mission.write(MissionEvent::End);
        }
    }
}
