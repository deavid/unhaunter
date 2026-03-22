mod auth;
mod connection;
mod procman;

use bevy::prelude::*;

pub(crate) fn app_setup(app: &mut App) {
    procman::app_setup(app);
    connection::app_setup(app);
    auth::app_setup(app);
}