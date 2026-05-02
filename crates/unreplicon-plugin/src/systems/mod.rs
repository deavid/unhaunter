pub mod debug;
pub mod ghost;
pub mod lobby;
pub mod players;
pub mod roles;

use bevy::prelude::*;
use bevy_replicon::prelude::{AuthMethod, RepliconPlugins, RepliconSharedPlugin};
use bevy_replicon_quinnet::RepliconQuinnetPlugins;
use uncommon_states_core::UIContextState;
use unreplicon_core::export_ext::RepliconExportSet;

pub(crate) fn app_setup(app: &mut App) {
    app.add_plugins((
        RepliconPlugins.set(RepliconSharedPlugin {
            auth_method: AuthMethod::Custom,
        }),
        RepliconQuinnetPlugins,
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
