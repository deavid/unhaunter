use bevy::prelude::*;
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientId, ClientMessageAppExt, ConnectedClient, FromClient, Replicated,
    ServerState,
};
use bevy_replicon::shared::backend::connected_client::NetworkId as RepliconNetworkId;
use unnet_core::resources::{CurrentMapSeed, LobbyData, LocalPlayer};
use unreplicon_core::components::{LobbyInfo, LobbyPlayerInfo, ServerAppState, SelectedMission};
use unreplicon_core::messages::{RequestSelectDifficulty, RequestSelectMap, RequestStartMission};
use untypes_core::cli::CliOptions;
use untypes_core::states::AppState;

pub(super) fn app_setup(app: &mut App) {
    // Register client → server messages
    app.add_client_message::<RequestSelectMap>(Channel::Ordered);
    app.add_client_message::<RequestSelectDifficulty>(Channel::Ordered);
    app.add_client_message::<RequestStartMission>(Channel::Ordered);

    // Register replicated components
    app.replicate::<LobbyInfo>();
    app.replicate::<ServerAppState>();
    app.replicate::<SelectedMission>();

    // Initialize resources that are referenced by lobby UI systems.
    // These are normally populated by unnet-plugin which is being removed.
    app.init_resource::<LobbyData>();
    app.init_resource::<LocalPlayer>();
    app.init_resource::<CurrentMapSeed>();
    app.init_resource::<unnet_core::resources::RoomIdentification>();

    // Observer: fires whenever a client entity gains ConnectedClient component.
    app.add_observer(on_client_connected);

    // Server-side lobby lifecycle
    app.add_systems(
        OnEnter(AppState::Lobby),
        setup_lobby_entity.run_if(in_state(ServerState::Running)),
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

/// Spawn the authoritative lobby state entity when the server enters Lobby.
///
/// For a listen server (Host mode) the host player (id=0, tint=0) is pre-added
/// to the players list and set as the room owner.
/// For a dedicated (headless) server the list starts empty; clients are added
/// via the `on_client_connected` observer.
fn setup_lobby_entity(mut commands: Commands, cli: Res<CliOptions>) {
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
        ServerAppState(AppState::Lobby),
    ));
    info!("Lobby entity spawned (headless={})", cli.is_headless());
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
    if q_lobby.is_empty() {
        return; // Not in a lobby phase — ignore
    }

    let entity = trigger.entity;
    let client_id_u64 = q_network_id
        .get(entity)
        .ok()
        .flatten()
        .map(|n| n.get())
        .unwrap_or(0);

    for mut lobby in q_lobby.iter_mut() {
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

/// Returns the `NetworkId` u64 for a `ClientId` by querying the component.
///
/// `ClientId::Server` ⇒ 0 (our sentinel for the host / listen-server player).
fn client_network_id(
    client_id: ClientId,
    q_network_id: &Query<Option<&RepliconNetworkId>>,
) -> u64 {
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
            // Update host_app_state in LobbyData so clients see "mission in progress".
        }
    }
}
