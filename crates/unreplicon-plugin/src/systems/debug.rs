use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_quinnet::client::QuinnetClient;
use bevy_quinnet::server::QuinnetServer;
use bevy_replicon::prelude::*;
use unreplicon_core::resources::{AuthorityRole, LocalPlayerRole};

#[derive(Resource, Default)]
struct ConnectionDebugTimer(Stopwatch);

pub(super) fn app_setup(app: &mut App) {
    app.init_resource::<ConnectionDebugTimer>();
    app.add_systems(Update, debug_connection_status);
}

fn debug_connection_status(
    time: Res<Time>,
    mut timer: ResMut<ConnectionDebugTimer>,
    client: Option<Res<QuinnetClient>>,
    server: Option<Res<QuinnetServer>>,
    authority: Option<Res<AuthorityRole>>,
    local: Option<Res<LocalPlayerRole>>,
    q_connected_clients: Query<&ConnectedClient>,
) {
    timer.0.tick(time.delta());
    if timer.0.elapsed_secs() < 10.0 {
        return;
    }
    timer.0.reset();

    if authority.is_some() {
        let count = q_connected_clients.iter().count();
        let server_status = if server.is_some() {
            "PRESENT"
        } else {
            "NOT_FOUND"
        };
        info!(
            "SERVER DEBUG: transport={} | connected_clients={}",
            server_status, count
        );
    }

    if local.is_some() {
        let client_status = if let Some(client) = client {
            if client.is_connected() {
                "CONNECTED".to_string()
            } else {
                "NOT_CONNECTED".to_string()
            }
        } else {
            "NOT_FOUND".to_string()
        };
        info!("CLIENT DEBUG: status={}", client_status);
    }
}
