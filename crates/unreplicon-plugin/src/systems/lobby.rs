use bevy::prelude::*;
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientId, ClientMessageAppExt, ConnectedClient, FromClient, Replicated,
    ServerState,
};
use unmapload_core::events::loadlevel::LoadLevelEvent;
use unprofile_core::profile::PlayerProfileData;
use unreplicon_core::components::{LobbyInfo, LobbyPlayerInfo, SelectedMission, ServerGamePhase};
use unreplicon_core::messages::{RequestSelectDifficulty, RequestSelectMap, RequestStartMission};
use unreplicon_core::ownership::OwnerId;
use unreplicon_core::resources::{ClientUuidMap, CurrentMapSeed, HostGone, LocalPlayer};
use untypes_core::roles::{AuthorityRole, LocalPlayerRole};
use untypes_core::states::{AppState, BootState};
use uuid::Uuid;

pub(super) fn app_setup(app: &mut App) {
    // Register client → server messages
    app.add_client_message::<RequestSelectMap>(Channel::Ordered);
    app.add_client_message::<RequestSelectDifficulty>(Channel::Ordered);
    app.add_client_message::<RequestStartMission>(Channel::Ordered);

    // Register replicated components
    app.replicate::<LobbyInfo>();
    app.replicate::<ServerGamePhase>();
    app.replicate::<SelectedMission>();

    // Initialize resources that are referenced by lobby UI systems.
    app.init_resource::<ClientUuidMap>();
    app.init_resource::<LocalPlayer>();
    app.init_resource::<CurrentMapSeed>();
    app.init_resource::<HostGone>();
    app.init_resource::<unreplicon_core::resources::RoomIdentification>();

    // Observer: fires whenever a client entity loses ConnectedClient on disconnect.
    app.add_observer(on_client_disconnected);

    app.add_systems(
        Update,
        process_newly_connected_clients.run_if(resource_exists::<AuthorityRole>),
    );

    // Two hooks so the transition to Lobby is caught regardless of which state
    // settles last.  On a dedicated server, ServerState::Running fires at startup
    // (before maps load) and BootState::Ready fires once maps are ready — both
    // can be the "later" one depending on timing.
    app.add_systems(
        OnEnter(ServerState::Running),
        auto_start_headless_lobby.run_if(in_state(BootState::Ready)),
    );
    app.add_systems(
        OnEnter(BootState::Ready),
        auto_start_headless_lobby.run_if(in_state(ServerState::Running)),
    );

    // Server-side lobby lifecycle
    app.add_systems(
        OnEnter(AppState::Lobby),
        setup_lobby_entity.run_if(resource_exists::<AuthorityRole>),
    );

    // Server-side: broadcast InGame state to clients when the mission starts.
    app.add_systems(
        OnEnter(AppState::InGame),
        set_server_state_ingame.run_if(resource_exists::<AuthorityRole>),
    );

    // Server-side message handlers
    app.add_systems(
        Update,
        (
            handle_request_select_map,
            handle_request_select_difficulty,
            handle_request_start_mission,
        )
            .run_if(resource_exists::<AuthorityRole>),
    );

    // Observe SimulationState::Ready to transition MissionLoading → InGame
    app.add_systems(
        Update,
        observe_simulation_ready_to_enter_game.run_if(in_state(AppState::MissionLoading)),
    );
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
) {
    // Log the actual values so we can see which conditions are or aren't met.
    info!(
        "auto_start_headless_lobby: authority={} local_player={} procman={}",
        authority.is_some(),
        local_player.is_some(),
        procman.is_some(),
    );
    // Dedicated server (authority, NO local player) or hub-less direct-connect authority.
    if authority.is_some() && local_player.is_none() && procman.is_none() {
        info!("Hub-less dedicated mode: auto-transitioning to AppState::Lobby");
        next_state.set(AppState::Lobby);
    }
}

/// Spawn (or reset) the authoritative lobby-state entity when the server enters Lobby.
fn setup_lobby_entity(
    mut q_existing: Query<(&mut LobbyInfo, &mut ServerGamePhase)>,
    mut commands: Commands,
    local_player: Option<Res<LocalPlayerRole>>,
    profile: Option<Res<PlayerProfileData>>,
) {
    if let Ok((mut lobby, mut game_phase)) = q_existing.single_mut() {
        // Re-entering Lobby after a mission: reset selection, signal state change.
        *game_phase = ServerGamePhase::Lobby;
        lobby.selected_map = None;
        info!("Lobby entity reset for new session");
        return;
    }

    // First-time spawn.
    let mut players = Vec::new();
    let mut leader_uuid = None;

    if local_player.is_some()
        && let Some(p) = profile
    {
        let uuid = p.installation_id;
        players.push(LobbyPlayerInfo {
            player_uuid: uuid,
            current_socket: None, // Local host player
            tint_color_index: 0,
            connected: true,
            nickname: None,
        });
        leader_uuid = Some(uuid);
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

/// Server: write `ServerGamePhase::InProgress` on the lobby entity when the server
/// enters `AppState::InGame`.
fn set_server_state_ingame(mut q: Query<&mut ServerGamePhase>) {
    for mut phase in q.iter_mut() {
        *phase = ServerGamePhase::InProgress;
    }
    info!("ServerGamePhase set to InProgress");
}

/// Observe `SimulationState::Ready` and transition `AppState::MissionLoading → AppState::InGame`.
fn observe_simulation_ready_to_enter_game(
    sim_state: Res<State<untypes_core::states::SimulationState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if *sim_state == untypes_core::states::SimulationState::Ready {
        next_app_state.set(AppState::InGame);
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
                let color_index = lobby.players.len() as u8;
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

                // If lobby has no leader, assign the first player.
                if lobby.leader_uuid.is_none() {
                    lobby.leader_uuid = Some(uuid);
                    info!("Player {} assigned as lobby leader", uuid);
                }
            }
        }
    }
}

fn on_client_disconnected(
    trigger: On<Remove, ConnectedClient>,
    mut q_lobby: Query<&mut LobbyInfo>,
    uuid_map: Res<ClientUuidMap>,
) {
    let client_id = ClientId::Client(trigger.entity);
    let Some(uuid) = client_uuid(client_id, &uuid_map) else {
        return;
    };

    for mut lobby in q_lobby.iter_mut() {
        if let Some(player) = lobby.players.iter_mut().find(|p| p.player_uuid == uuid) {
            player.connected = false;
            player.current_socket = None;
            info!("Player {} disconnected", uuid);
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
        let Some(sender_uuid) = client_uuid(msg.client_id, &uuid_map) else {
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
        }
    }
}
