pub mod debug;
pub mod ghost;
pub mod lobby;
pub mod players;
pub mod roles;

use bevy::prelude::*;
use bevy_replicon::prelude::RepliconPlugins;
use bevy_replicon_renet::RepliconRenetPlugins;

pub(crate) fn app_setup(app: &mut App) {
    let transport_config = app
        .world()
        .resource::<unreplicon_transport::resources::TransportConfig>()
        .clone();
    let procman_config = app
        .world()
        .resource::<unreplicon_transport::resources::ProcManConfig>()
        .clone();

    app.add_plugins((
        RepliconPlugins,
        RepliconRenetPlugins,
        unreplicon_transport::plugin::UnrepliconTransportPlugin {
            transport_config,
            procman_config,
        },
    ));
    lobby::app_setup(app);
    players::app_setup(app);
    ghost::app_setup(app);
    debug::app_setup(app);
    roles::app_setup(app);
}
