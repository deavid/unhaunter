mod auth;
mod connection;
mod procman;

use bevy::prelude::*;

use crate::plugin::NetworkRole;

pub(crate) fn app_setup(app: &mut App, network_role: NetworkRole) {
    procman::app_setup(app);
    connection::app_setup(app);
    auth::app_setup(app, network_role);
}
