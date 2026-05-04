pub mod debug;
pub mod ghost;
pub mod lobby;
pub mod players;
pub mod roles;

use bevy::prelude::*;
use bevy_replicon::prelude::AuthMethod;
use bevy_replicon::prelude::RepliconPlugins;
use bevy_replicon::prelude::RepliconSharedPlugin;
use bevy_replicon_renet2::RepliconRenetPlugins;
use uncommon_states_core::UIContextState;
use unreplicon_core::export_ext::RepliconExportSet;

const USE_CUSTOM_AUTH: bool = true;

pub(crate) fn app_setup(app: &mut App) {
    let auth_method = if USE_CUSTOM_AUTH {
        AuthMethod::Custom
    } else {
        AuthMethod::ProtocolCheck
    };

    app.add_plugins((RepliconPlugins.set(RepliconSharedPlugin { auth_method }),));

    let transport_config = app
        .world()
        .resource::<unreplicon_transport::resources::TransportConfig>()
        .clone();
    let procman_config = app
        .world()
        .resource::<unreplicon_transport::resources::ProcManConfig>()
        .clone();

    app.add_plugins((
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
