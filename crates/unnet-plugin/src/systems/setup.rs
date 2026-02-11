use super::client_input::{
    client_process_pending_map, client_request_grab_system, client_send_input_system,
    client_sync_intended_gear_state,
};
use super::client_sync::{client_apply_snapshots_system, delayed_despawn_system};
use super::connection::{
    autostart_net_game, client_connection_monitor_system, client_heartbeat_system,
    client_lobby_state_handler, client_start_mission_handler, client_state_bootstrap_system,
    handshake_handler_system, host_handle_disconnects_system, host_liveness_system,
    host_process_heartbeats_system, host_status_updater_system, lobby_broadcast_state_system,
    network_io_system, session_roster_system, startup_network_system,
};
use super::host_input::host_apply_input_system;
use super::host_sync::{host_send_snapshots_system, host_send_summary_system};
use bevy::prelude::*;
use untypes_core::states::AppState;

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Startup, startup_network_system);
    app.add_systems(OnEnter(AppState::MainMenu), autostart_net_game);
    app.init_resource::<unnet_core::resources::CurrentMapSeed>();
    app.add_systems(OnEnter(AppState::Summary), host_send_summary_system);

    app.add_systems(Update, (lobby_broadcast_state_system, host_liveness_system));
    app.add_systems(
        FixedUpdate,
        (client_heartbeat_system, host_status_updater_system),
    );

    app.add_systems(
        Update,
        client_process_pending_map.run_if(
            in_state(AppState::MainMenu)
                .or(in_state(AppState::Lobby))
                .or(in_state(AppState::Loading)),
        ),
    );

    app.add_systems(
        PreUpdate,
        (
            network_io_system,
            host_process_heartbeats_system,
            host_handle_disconnects_system,
            handshake_handler_system,
            session_roster_system,
            client_lobby_state_handler,
            client_state_bootstrap_system,
            client_start_mission_handler,
            host_apply_input_system.run_if(in_state(AppState::InGame)),
        )
            .chain(),
    );

    app.add_systems(
        Update,
        (
            host_send_snapshots_system,
            client_request_grab_system,
            client_sync_intended_gear_state,
            client_send_input_system,
            client_apply_snapshots_system,
            client_connection_monitor_system,
            delayed_despawn_system,
        )
            .chain()
            .after(unplayer_core::PlayerInputSet)
            .run_if(in_state(AppState::InGame)),
    );
}
