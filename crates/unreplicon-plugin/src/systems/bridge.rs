use std::str::FromStr;

use bevy::prelude::*;
use bevy_renet::netcode::NetcodeClientTransport;
use bevy_replicon::prelude::ServerState;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unmapload_core::events::loadlevel::LoadLevelEvent;
use unreplicon_core::components::{LobbyInfo, SelectedMission, ServerGamePhase};
use unreplicon_core::network_id::NetworkId as GameNetworkId;
use unreplicon_core::resources::{CurrentMapSeed, LobbyData, LobbyPlayer, LocalPlayer, RoomOwner};
use untypes_core::cli::{CliOptions, NetMode};
use untypes_core::difficulty::Difficulty;
use untypes_core::states::AppState;

pub(super) fn app_setup(app: &mut App) {
    app.add_systems(PostStartup, set_local_player_system);
    app.add_systems(
        Update,
        (bridge_lobby_info_system, react_to_server_game_phase)
            .run_if(not(in_state(ServerState::Running))),
    );
    // Join-mode clients: on MainMenu entry, skip the menu and go directly to Lobby.
    app.add_systems(
        OnEnter(AppState::MainMenu),
        auto_join_to_lobby.run_if(not(in_state(ServerState::Running))),
    );
    // On re-entering the Lobby screen, unconditionally sync LobbyData from LobbyInfo.
    app.add_systems(
        OnEnter(AppState::Lobby),
        rehydrate_lobby_data_on_enter.run_if(not(in_state(ServerState::Running))),
    );
    app.add_observer(on_selected_mission_added);
}

/// Initialise `LocalPlayer` based on the network mode.
///
/// - `Offline`: leaves the default `LocalPlayer(None)`.
/// - `Host`: sets `LocalPlayer(NetworkId(0))` — 0 is our sentinel for the server/host.
/// - `Join`: reads the client_id from the netcode transport (set during `Startup`).
fn set_local_player_system(
    cli: Res<CliOptions>,
    transport: Option<Res<NetcodeClientTransport>>,
    mut commands: Commands,
) {
    match &cli.net_mode {
        NetMode::Offline => {
            // LocalPlayer(None) from Default init is correct for offline play.
        }
        NetMode::Host { .. } => {
            // Host is always the first player; use id=0 as the server-player sentinel.
            commands.insert_resource(LocalPlayer(Some(GameNetworkId(0))));
        }
        NetMode::Join { .. } => {
            if let Some(t) = transport {
                let client_id = t.client_id();
                commands.insert_resource(LocalPlayer(Some(GameNetworkId(client_id))));
                info!("LocalPlayer set to NetworkId({}) for Join mode", client_id);
            } else {
                warn!("Join mode but NetcodeClientTransport is absent; LocalPlayer not set");
            }
        }
    }
}

/// Bridge: reads the replicated `LobbyInfo` component and writes `LobbyData` + `RoomOwner`.
///
/// Run condition: only on nodes where the server is NOT running (i.e. dedicated clients
/// and listen-server hosts where the server-side lobby.rs already owns `LobbyData`).
/// Actually for Host mode the server spawns LobbyInfo locally, so this bridge is still
/// beneficial to keep LobbyData in sync. We run it whenever LobbyInfo has changed.
///
/// FIXME Phase 3: consider a cleaner split between server-authoritative mutation and
/// client-side bridge.
fn bridge_lobby_info_system(
    q_lobby: Query<&LobbyInfo, Changed<LobbyInfo>>,
    mut lobby_data: ResMut<LobbyData>,
    mut commands: Commands,
) {
    let Ok(lobby_info) = q_lobby.single() else {
        return;
    };

    info!(
        "Bridging LobbyInfo: {} players, owner={}, map={:?}",
        lobby_info.players.len(),
        lobby_info.owner_client_id,
        lobby_info.selected_map
    );

    // Rebuild the player list
    lobby_data.players = lobby_info
        .players
        .iter()
        .map(|p| LobbyPlayer {
            id: GameNetworkId(p.client_id),
            tint_color_index: p.tint_color_index,
            connected: p.connected,
            nickname: p.nickname.clone(),
        })
        .collect();

    lobby_data.selected_map = lobby_info.selected_map.clone();
    lobby_data.selected_difficulty = lobby_info.selected_difficulty.clone();

    // Update RoomOwner resource so unlobby-plugin can gate UI buttons
    commands.insert_resource(RoomOwner(GameNetworkId(lobby_info.owner_client_id)));
}

/// Observer: fires when `SelectedMission` is added to any entity.
///
/// This triggers the level-loading pipeline on non-headless nodes (clients and listen-
/// server hosts). On a headless dedicated server we skip UI/rendering concerns.
fn on_selected_mission_added(
    trigger: On<Insert, SelectedMission>,
    q_mission: Query<&SelectedMission>,
    cli: Res<CliOptions>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ev_load: MessageWriter<LoadLevelEvent>,
    mut current_map_seed: ResMut<CurrentMapSeed>,
    mut current_difficulty: ResMut<CurrentDifficulty>,
) {
    if cli.is_headless() {
        // Dedicated server: no level loading needed from the client side.
        return;
    }

    let entity = trigger.entity;
    let Ok(mission) = q_mission.get(entity) else {
        error!(
            "on_selected_mission_added: entity {:?} has no SelectedMission",
            entity
        );
        return;
    };

    info!(
        "Mission starting — map={}, difficulty={}, seed={}",
        mission.map_path, mission.difficulty_id, mission.map_seed
    );

    current_map_seed.0 = mission.map_seed;

    if let Ok(diff) = Difficulty::from_str(&mission.difficulty_id) {
        *current_difficulty = CurrentDifficulty::new(diff);
    } else {
        warn!(
            "Unknown difficulty id '{}'; keeping current setting",
            mission.difficulty_id
        );
    }

    ev_load.write(LoadLevelEvent {
        map_filepath: mission.map_path.clone(),
    });
    info!("Transitioning to AppState::Loading (mission start)");
    next_state.set(AppState::Loading);
}

/// Client: react to `ServerGamePhase` changes to drive guarded state transitions.
///
/// - `Lobby`: if the client is currently in `InGame` or `Summary`, return to the Lobby
///   screen (e.g., everyone is kicked back after a mission ends).
/// - `InProgress`: no-op here — `on_selected_mission_added` drives the InGame transition.
/// - `Ended`: if the client is in `InGame`, move to `Summary`.
///
/// The guards prevent the system from acting while the client is in `Loading` or on
/// the `MainMenu`, which eliminates the freeze-on-join race that plagued the old
/// `follow_server_app_state` implementation.
fn react_to_server_game_phase(
    q_phase: Query<&ServerGamePhase, Changed<ServerGamePhase>>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(phase) = q_phase.single() else {
        return;
    };
    let current = *app_state.get();
    match phase {
        ServerGamePhase::Lobby => {
            if matches!(current, AppState::InGame | AppState::Summary) {
                info!("ServerGamePhase::Lobby received; returning client to Lobby");
                next_state.set(AppState::Lobby);
            }
        }
        ServerGamePhase::InProgress => {
            // Handled by on_selected_mission_added observer.
        }
        ServerGamePhase::Ended => {
            if current == AppState::InGame {
                info!("ServerGamePhase::Ended received; transitioning client to Summary");
                next_state.set(AppState::Summary);
            }
        }
    }
}

/// Join-mode client: skip `MainMenu` and go directly to `AppState::Lobby`.
///
/// Registered on `OnEnter(AppState::MainMenu)` with a `not(in_state(ServerState::Running))`
/// guard so it only fires on the client side. This replaces the old approach of reacting
/// to `ServerAppState::Lobby` from `follow_server_app_state`, which raced with the asset
/// loader and froze the game.
fn auto_join_to_lobby(cli: Res<CliOptions>, mut next_state: ResMut<NextState<AppState>>) {
    if matches!(cli.net_mode, NetMode::Join { .. }) {
        info!("auto_join_to_lobby: Join mode detected on MainMenu; transitioning to Lobby");
        next_state.set(AppState::Lobby);
    }
}

/// On enter to `AppState::Lobby`, unconditionally sync `LobbyData` from the current
/// `LobbyInfo` component, bypassing the `Changed<LobbyInfo>` filter.
///
/// This ensures that when a client re-enters the lobby screen (e.g., after ESC → back)
/// they immediately see the current player list and map selection rather than an empty UI.
fn rehydrate_lobby_data_on_enter(
    q_lobby: Query<&LobbyInfo>,
    mut lobby_data: ResMut<LobbyData>,
    mut commands: Commands,
) {
    let Ok(lobby_info) = q_lobby.single() else {
        return;
    };
    info!(
        "rehydrate_lobby_data_on_enter: {} players, owner={}",
        lobby_info.players.len(),
        lobby_info.owner_client_id
    );
    lobby_data.players = lobby_info
        .players
        .iter()
        .map(|p| LobbyPlayer {
            id: GameNetworkId(p.client_id),
            tint_color_index: p.tint_color_index,
            connected: p.connected,
            nickname: p.nickname.clone(),
        })
        .collect();
    lobby_data.selected_map = lobby_info.selected_map.clone();
    lobby_data.selected_difficulty = lobby_info.selected_difficulty.clone();
    commands.insert_resource(RoomOwner(GameNetworkId(lobby_info.owner_client_id)));
}
