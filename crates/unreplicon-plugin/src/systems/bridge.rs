use bevy::prelude::*;
use bevy_renet::netcode::NetcodeClientTransport;
use bevy_replicon::prelude::ServerState;
use unreplicon_core::components::LobbyInfo;
use unreplicon_core::network_id::NetworkId as GameNetworkId;
use unreplicon_core::resources::{LobbyData, LobbyPlayer, LocalPlayer, RoomOwner};
use untypes_core::cli::{CliOptions, NetMode};
use untypes_core::states::AppState;

pub(super) fn app_setup(app: &mut App) {
    app.add_systems(PostStartup, set_local_player_system);
    app.add_systems(
        Update,
        bridge_lobby_info_system.run_if(not(in_state(ServerState::Running))),
    );
    // On re-entering the Lobby screen, unconditionally sync LobbyData from LobbyInfo.
    app.add_systems(
        OnEnter(AppState::Lobby),
        rehydrate_lobby_data_on_enter.run_if(not(in_state(ServerState::Running))),
    );
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
