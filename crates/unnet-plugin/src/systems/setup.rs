use super::client_input::{
    client_process_pending_map, client_request_grab_system, client_send_input_system,
    client_sync_intended_gear_state,
};
use super::client_sync::{client_apply_snapshots_system, delayed_despawn_system};
use super::connection::{
    autostart_net_game, client_connection_monitor_system, client_heartbeat_system,
    client_lobby_state_handler, client_start_mission_handler, client_state_bootstrap_system,
    connect_to_server_system, handshake_handler_system, headless_room_reclaim_system,
    headless_summary_reset_system, host_handle_disconnects_system, host_liveness_system,
    host_process_heartbeats_system, host_status_updater_system, idle_timeout_system,
    lobby_broadcast_state_system, network_io_system, session_roster_system, startup_network_system,
};
use super::procman::{
    dynamic_tick_rate_system, procman_player_events_system, procman_state_sync_system,
    setup_procman_system, update_procman_system,
};
use super::host_input::host_apply_input_system;
use super::host_sync::{host_send_snapshots_system, host_send_summary_system};
use bevy::prelude::*;
use untypes_core::states::AppState;

pub(crate) fn app_setup(app: &mut App) {
    let is_headless = app
        .world()
        .get_resource::<untypes_core::cli::CliOptions>()
        .map(|cli| cli.dedicated)
        .unwrap_or(false);

    app.init_resource::<unnet_core::resources::CurrentMapSeed>();
    app.add_systems(OnEnter(AppState::Summary), host_send_summary_system);
    app.add_systems(Update, headless_summary_reset_system);
    if is_headless {
        app.add_systems(Update, (idle_timeout_system, headless_room_reclaim_system));
        app.add_systems(Startup, setup_procman_system);
        app.add_systems(
            OnEnter(AppState::Lobby),
            (startup_network_system, autostart_net_game),
        );
        app.add_systems(
            Update,
            (
                update_procman_system,
                procman_state_sync_system,
                procman_player_events_system,
                dynamic_tick_rate_system,
            ),
        );
    } else {
        app.add_systems(Startup, startup_network_system);
        app.add_systems(OnEnter(AppState::MainMenu), autostart_net_game);
    }

    app.add_systems(Update, (lobby_broadcast_state_system, host_liveness_system));
    app.add_systems(
        FixedUpdate,
        (client_heartbeat_system, host_status_updater_system),
    );

    if !is_headless {
        app.add_systems(
            Update,
            client_process_pending_map.run_if(
                in_state(AppState::MainMenu)
                    .or(in_state(AppState::Lobby))
                    .or(in_state(AppState::Loading)),
            ),
        );
    }

    app.add_systems(
        PreUpdate,
        (
            connect_to_server_system,
            network_io_system,
            host_process_heartbeats_system,
            host_handle_disconnects_system,
            handshake_handler_system,
            session_roster_system,
            client_lobby_state_handler,
            client_state_bootstrap_system,
            client_start_mission_handler,
            host_apply_input_system
                .run_if(in_state(AppState::InGame).or(in_state(AppState::Lobby))),
        )
            .chain(),
    );

    if is_headless {
        app.add_systems(
            Update,
            (host_send_snapshots_system, delayed_despawn_system)
                .chain()
                .after(unplayer_core::PlayerInputSet)
                .after(unplayer_core::authoritative::PlayerAuthoritativeLogicSet)
                .run_if(in_state(AppState::InGame)),
        );
    } else {
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
                .after(unplayer_core::authoritative::PlayerAuthoritativeLogicSet)
                .run_if(in_state(AppState::InGame)),
        );
    }
}
