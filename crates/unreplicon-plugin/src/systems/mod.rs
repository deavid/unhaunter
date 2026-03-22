pub(crate) mod debug;
pub(crate) mod ghost;
pub(crate) mod lobby;
pub mod players;
pub(crate) mod roles;

use bevy::prelude::*;
use bevy_replicon::prelude::RepliconPlugins;
use bevy_replicon_renet::RepliconRenetPlugins;

pub(crate) fn app_setup(app: &mut App) {
    app.add_plugins((
        RepliconPlugins,
        RepliconRenetPlugins,
        unreplicon_transport::plugin::UnrepliconTransportPlugin,
    ));
    lobby::app_setup(app);
    players::app_setup(app);
    ghost::app_setup(app);
    debug::app_setup(app);
    roles::app_setup(app);
}
