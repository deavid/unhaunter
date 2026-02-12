use super::systems;
use bevy::prelude::*;
use unnet_core::messages::NetworkDataEvent;

pub struct UnhaunterNetPlugin;

impl Plugin for UnhaunterNetPlugin {
    fn build(&self, app: &mut App) {
        crate::metrics::register_all(app);
        app.init_resource::<crate::resources::NetworkConn>();
        app.init_resource::<crate::resources::PlayerRegistry>();
        app.init_resource::<unnet_core::resources::LocalPlayer>();
        app.init_resource::<unnet_core::resources::ChangedTiles>();
        app.init_resource::<unnet_core::resources::MissionEndRequested>();
        app.init_resource::<unnet_core::resources::HostGone>();
        app.init_resource::<unnet_core::resources::LobbyData>();
        app.init_resource::<crate::resources::PendingMapLoad>();
        app.add_message::<NetworkDataEvent>();
        app.add_message::<unnet_core::messages::NetworkDisconnectEvent>();
        app.add_message::<unnet_core::messages::PlayerJoinedEvent>();
        app.add_message::<unnet_core::messages::PlayerDiedEvent>();
        app.add_message::<unnet_core::messages::SendNetworkMessage>();
        app.add_message::<unnet_core::messages::TransientEvent>();

        systems::setup::app_setup(app);
    }
}
