use std::str::FromStr;

use bevy::prelude::*;
use bevy_renet::netcode::NetcodeClientTransport;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unmapload_core::events::loadlevel::LoadLevelEvent;
use unnet_core::network_id::NetworkId as GameNetworkId;
use unnet_core::resources::{CurrentMapSeed, LobbyData, LobbyPlayer, LocalPlayer, RoomOwner};
use unreplicon_core::components::{LobbyInfo, SelectedMission};
use untypes_core::cli::{CliOptions, NetMode};
use untypes_core::difficulty::Difficulty;
use untypes_core::states::AppState;

pub(super) fn app_setup(app: &mut App) {
    app.add_systems(PostStartup, set_local_player_system);
    app.add_systems(Update, bridge_lobby_info_system);
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
    lobby_data.host_app_state = None; // updated separately when SelectedMission arrives

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
        error!("on_selected_mission_added: entity {:?} has no SelectedMission", entity);
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
    next_state.set(AppState::Loading);
}
