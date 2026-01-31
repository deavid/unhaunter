use crate::events::NetworkDataEvent;
use crate::resources::NetworkConn;
use crate::systems::{
    autostart_net_game, client_apply_snapshots_system, client_send_input_system,
    handshake_handler_system, host_apply_input_system, host_send_snapshots_system,
    network_io_system, startup_network_system,
};
use bevy::prelude::*;
use untypes_core::states::AppState;

pub struct UnhaunterNetPlugin;

impl Plugin for UnhaunterNetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkConn>();
        app.add_message::<NetworkDataEvent>();

        app.add_systems(Startup, startup_network_system);
        app.add_systems(OnEnter(AppState::MainMenu), autostart_net_game);

        app.add_systems(
            PreUpdate,
            (
                network_io_system,
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
