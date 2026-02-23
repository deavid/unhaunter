mod auth;
mod bridge;
mod connection;
pub(crate) mod lobby;
pub(crate) mod procman;

use bevy::prelude::*;
use bevy_replicon::prelude::RepliconPlugins;
use bevy_replicon_renet::RepliconRenetPlugins;

pub(crate) fn app_setup(app: &mut App) {
    app.add_plugins((RepliconPlugins, RepliconRenetPlugins));
    procman::app_setup(app);
    connection::app_setup(app);
    auth::app_setup(app);
    lobby::app_setup(app);
    bridge::app_setup(app);
}
