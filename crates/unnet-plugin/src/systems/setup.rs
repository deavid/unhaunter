use super::client_input::{
    client_process_pending_map, client_request_grab_system, client_send_input_system,
    client_sync_intended_gear_state,
};
use super::client_sync::{client_apply_snapshots_system, delayed_despawn_system};
use super::connection::{
    autostart_net_game, client_connection_monitor_system, handshake_handler_system,
    host_handle_disconnects_system, network_io_system, startup_network_system,
};
use super::host_input::host_apply_input_system;
use super::host_sync::{host_send_snapshots_system, host_send_summary_system};
use bevy::prelude::*;
use untypes_core::states::AppState;

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Startup, startup_network_system);
    app.add_systems(OnEnter(AppState::MainMenu), autostart_net_game);
    app.add_systems(OnEnter(AppState::Summary), host_send_summary_system);

    app.add_systems(
        Update,
        client_process_pending_map.run_if(in_state(AppState::MainMenu)),
    );

    app.add_systems(
        PreUpdate,
        (
            network_io_system,
            host_handle_disconnects_system,
            handshake_handler_system,
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
