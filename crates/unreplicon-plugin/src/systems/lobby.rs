use bevy::prelude::*;
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientId, ClientMessageAppExt, ConnectedClient, FromClient, Replicated,
    ServerState,
};
use bevy_replicon::shared::backend::connected_client::NetworkId as RepliconNetworkId;
use unmapload_core::events::loadlevel::LoadLevelEvent;
use unreplicon_core::components::{LobbyInfo, LobbyPlayerInfo, SelectedMission, ServerGamePhase};
use unreplicon_core::messages::{RequestSelectDifficulty, RequestSelectMap, RequestStartMission};
use unreplicon_core::resources::{CurrentMapSeed, HostGone, LobbyData, LocalPlayer};
use untypes_core::cli::CliOptions;
use untypes_core::states::AppState;

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
    // These are normally populated by unnet-plugin which is being removed.
    app.init_resource::<LobbyData>();
    app.init_resource::<LocalPlayer>();
    app.init_resource::<CurrentMapSeed>();
    app.init_resource::<HostGone>();
    app.init_resource::<unreplicon_core::resources::RoomIdentification>();

    // Observer: fires whenever a client entity gains ConnectedClient component.
    app.add_observer(on_client_connected);
    // Observer: fires whenever a client entity loses ConnectedClient on disconnect.
    app.add_observer(on_client_disconnected);

    // Server-side: auto-start lobby in hub-less dedicated mode.
    // Uses OnEnter(ServerState::Running) because ServerState starts as Stopped and
    // is transitioned to Running by the bevy_replicon_renet backend in the first
    // PreUpdate. PostStartup fires before that, so run_if(in_state(ServerState::Running))
    // on PostStartup would always be false.
    app.add_systems(OnEnter(ServerState::Running), auto_start_headless_lobby);

    // Server-side lobby lifecycle
    app.add_systems(
        OnEnter(AppState::Lobby),
        setup_lobby_entity.run_if(in_state(ServerState::Running)),
    );

    // Server-side: broadcast InGame state to clients when the mission starts.
    app.add_systems(
        OnEnter(AppState::InGame),
        set_server_state_ingame.run_if(in_state(ServerState::Running)),
    );

    // Server-side message handlers
    app.add_systems(
        Update,
        (
            handle_request_select_map,
            handle_request_select_difficulty,
            handle_request_start_mission,
        )
            .run_if(in_state(ServerState::Running)),
    );
}

/// Server: In hub-less dedicated mode, transition to Lobby immediately.
///
/// When running under a process manager (hub mode), the transition is managed
/// by procman via `AssignRoom`. In standalone mode, we must start the lobby
/// ourselves or clients will have nothing to connect to.
fn auto_start_headless_lobby(
    cli: Res<CliOptions>,
    procman: Option<Res<crate::systems::procman::ProcManChannel>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if cli.is_headless() && procman.is_none() {
        info!("Hub-less dedicated mode: auto-transitioning to AppState::Lobby");
        next_state.set(AppState::Lobby);
    }
}

/// Spawn (or reset) the authoritative lobby-state entity when the server enters Lobby.
///
/// On the very first entry (e.g., server startup) a fresh entity is spawned.
/// On subsequent re-entries (e.g., after a mission ends and the server returns to
/// Lobby) the existing entity is updated in-place so that bevy_replicon propagates
/// the `Changed<ServerGamePhase>` to connected clients, triggering
/// `react_to_server_game_phase` on each client and returning them to the Lobby screen.
///
/// Re-using the entity also avoids the duplicate-entity problem that would arise
/// from spawning unconditionally: that system uses `q.single()` and would panic if
/// two `ServerGamePhase` entities coexisted.
fn setup_lobby_entity(
    mut q_existing: Query<(&mut LobbyInfo, &mut ServerGamePhase)>,
    mut commands: Commands,
    cli: Res<CliOptions>,
) {
    if let Ok((mut lobby, mut game_phase)) = q_existing.single_mut() {
        // Re-entering Lobby after a mission: reset selection, signal state change.
        *game_phase = ServerGamePhase::Lobby;
        lobby.selected_map = None;
        info!(
            "Lobby entity reset for new session (headless={})",
            cli.is_headless()
        );
        return;
    }

    // First-time spawn.
    let players = if cli.is_headless() {
        Vec::new()
    } else {
        // Host (listen server): add local player with id = 0
        vec![LobbyPlayerInfo {
            client_id: 0,
            tint_color_index: 0,
            connected: true,
            nickname: None,
        }]
    };

    commands.spawn((
        Replicated,
        LobbyInfo {
            players,
            selected_map: None,
            selected_difficulty: "standard-challenge".to_string(),
            owner_client_id: 0, // 0 = server / host
        },
        ServerGamePhase::Lobby,
    ));
    info!(
        "Lobby entity spawned (headless={}, owner=0)",
        cli.is_headless()
    );
}

/// Server: write `ServerGamePhase::InProgress` on the lobby entity when the server
/// enters `AppState::InGame`.
///
/// Clients react to this via `react_to_server_game_phase` — but the InProgress
/// transition itself is driven by the `on_selected_mission_added` observer, so the
/// client does not navigate on this signal alone. Writing it here keeps the phase
/// accurate for any system that queries current game phase.
fn set_server_state_ingame(mut q: Query<&mut ServerGamePhase>) {
    for mut phase in q.iter_mut() {
        *phase = ServerGamePhase::InProgress;
    }
    info!("ServerGamePhase set to InProgress");
}

/// Observer: triggered whenever `ConnectedClient` is added to an entity.
///
/// Appends the new player to all existing `LobbyInfo` entities.
/// No-ops if there is no active lobby (avoids firing during non-lobby states).
fn on_client_connected(
    trigger: On<Insert, ConnectedClient>,
    q_network_id: Query<Option<&RepliconNetworkId>>,
    mut q_lobby: Query<&mut LobbyInfo>,
) {
    let entity = trigger.entity;
    let client_id_u64 = q_network_id
        .get(entity)
        .ok()
        .flatten()
        .map(|n| n.get())
        .unwrap_or(0);

    if q_lobby.is_empty() {
        debug!(
            "on_client_connected: client {} connected but no lobby entity exists; ignoring",
            client_id_u64
        );
        return; // Not in a lobby phase — ignore
    }

    for mut lobby in q_lobby.iter_mut() {
        // If this is the first player joining a headless server, make them the owner.
        if lobby.players.is_empty() && lobby.owner_client_id == 0 {
            lobby.owner_client_id = client_id_u64;
            info!("First client {} assigned as room owner", client_id_u64);
        }

        let color_index = lobby.players.len() as u8;
        lobby.players.push(LobbyPlayerInfo {
            client_id: client_id_u64,
            tint_color_index: color_index,
            connected: true,
            nickname: None,
        });
        info!(
            "Client {} joined lobby (tint={})",
            client_id_u64, color_index
        );
    }
}

fn on_client_disconnected(
    trigger: On<Remove, ConnectedClient>,
    q_network_id: Query<Option<&RepliconNetworkId>>,
    mut q_lobby: Query<&mut LobbyInfo>,
) {
    let entity = trigger.entity;
    let client_id_u64 = q_network_id
        .get(entity)
        .ok()
        .flatten()
        .map(|n| n.get())
        .unwrap_or(0);

    for mut lobby in q_lobby.iter_mut() {
        lobby.players.retain(|p| p.client_id != client_id_u64);
        info!("Client {} removed from lobby player list", client_id_u64);

        // If the owner left, assign ownership to the next player in the list.
        if lobby.owner_client_id == client_id_u64 {
            lobby.owner_client_id = lobby.players.first().map(|p| p.client_id).unwrap_or(0);
            info!(
                "Owner disconnected; new owner_client_id={}",
                lobby.owner_client_id
            );
        }
    }
}

/// Returns the `NetworkId` u64 for a `ClientId` by querying the component.
///
/// `ClientId::Server` ⇒ 0 (our sentinel for the host / listen-server player).
fn client_network_id(client_id: ClientId, q_network_id: &Query<Option<&RepliconNetworkId>>) -> u64 {
    match client_id {
        ClientId::Server => 0,
        ClientId::Client(entity) => q_network_id
            .get(entity)
            .ok()
            .flatten()
            .map(|n| n.get())
            .unwrap_or(0),
    }
}

fn handle_request_select_map(
    mut reader: MessageReader<FromClient<RequestSelectMap>>,
    mut q_lobby: Query<&mut LobbyInfo>,
    q_network_id: Query<Option<&RepliconNetworkId>>,
) {
    for msg in reader.read() {
        let sender_id = client_network_id(msg.client_id, &q_network_id);
        for mut lobby in q_lobby.iter_mut() {
            if sender_id != lobby.owner_client_id {
                warn!(
                    "RequestSelectMap from non-owner client {} (owner={}); ignored",
                    sender_id, lobby.owner_client_id
                );
                continue;
            }
            info!("Map selected: {}", msg.map_filepath);
            lobby.selected_map = Some(msg.map_filepath.clone());
        }
    }
}

fn handle_request_select_difficulty(
    mut reader: MessageReader<FromClient<RequestSelectDifficulty>>,
    mut q_lobby: Query<&mut LobbyInfo>,
    q_network_id: Query<Option<&RepliconNetworkId>>,
) {
    for msg in reader.read() {
        let sender_id = client_network_id(msg.client_id, &q_network_id);
        for mut lobby in q_lobby.iter_mut() {
            if sender_id != lobby.owner_client_id {
                warn!(
                    "RequestSelectDifficulty from non-owner client {}; ignored",
                    sender_id
                );
                continue;
            }
            info!("Difficulty selected: {}", msg.difficulty_id);
            lobby.selected_difficulty = msg.difficulty_id.clone();
        }
    }
}

fn handle_request_start_mission(
    mut reader: MessageReader<FromClient<RequestStartMission>>,
    mut q_lobby: Query<&mut LobbyInfo>,
    q_network_id: Query<Option<&RepliconNetworkId>>,
    mut commands: Commands,
    mut ev_load: MessageWriter<LoadLevelEvent>,
) {
    for msg in reader.read() {
        let sender_id = client_network_id(msg.client_id, &q_network_id);
        for lobby in q_lobby.iter_mut() {
            if sender_id != lobby.owner_client_id {
                warn!(
                    "RequestStartMission from non-owner client {}; ignored",
                    sender_id
                );
                continue;
            }
            let Some(ref map_path) = lobby.selected_map.clone() else {
                warn!("StartMission requested but no map is selected");
                continue;
            };
            if map_path.is_empty() {
                warn!("StartMission requested with empty map path");
                continue;
            }
            info!(
                "Mission starting: map={}, difficulty={}, seed={}",
                map_path, lobby.selected_difficulty, msg.map_seed
            );
            commands.spawn((
                Replicated,
                SelectedMission {
                    map_path: map_path.clone(),
                    map_seed: msg.map_seed,
                    difficulty_id: lobby.selected_difficulty.clone(),
                },
            ));
            // Load the level server-side so the server enters AppState::InGame,
            // enabling player/gear spawning and replication to clients.
            ev_load.write(LoadLevelEvent {
                map_filepath: map_path.clone(),
            });
        }
    }
}
