use crate::systems::{
    autostart_net_game, client_apply_snapshots_system, client_process_pending_map,
    client_send_input_system, handshake_handler_system, host_apply_input_system,
    host_handle_disconnects_system, host_send_snapshots_system, host_send_summary_system,
    network_io_system, startup_network_system,
};
use bevy::prelude::*;
use unnet_core::messages::NetworkDataEvent;
use untypes_core::states::AppState;

pub struct UnhaunterNetPlugin;

impl Plugin for UnhaunterNetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::resources::NetworkConn>();
        app.init_resource::<unnet_core::resources::LocalPlayer>();
        app.init_resource::<crate::resources::PendingMapLoad>();
        app.add_message::<NetworkDataEvent>();
        app.add_message::<unnet_core::messages::NetworkDisconnectEvent>();
        app.add_message::<unnet_core::messages::TransientEvent>();

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
                client_send_input_system,
                client_apply_snapshots_system,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        );
    }
}
