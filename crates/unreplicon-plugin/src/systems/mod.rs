pub mod debug;
pub mod ghost;
pub mod lobby;
pub mod players;
pub mod roles;

use bevy::prelude::*;
use bevy_replicon::prelude::RepliconPlugins;
use bevy_replicon_renet2::RepliconRenetPlugins;
use uncommon_states_core::UIContextState;
use unreplicon_core::export_ext::RepliconExportSet;

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
    app.configure_sets(
        Update,
        RepliconExportSet.run_if(in_state(UIContextState::InGame)),
    );
    lobby::app_setup(app);
    players::app_setup(app);
    ghost::app_setup(app);
    debug::app_setup(app);
    roles::app_setup(app);
}
