use std::net::{SocketAddr, UdpSocket};
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use bevy_renet::netcode::{
    ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication,
    ServerConfig,
};
use bevy_renet::renet::ConnectionConfig;
use bevy_renet::{RenetClient, RenetServer};
use bevy_replicon::prelude::RepliconChannels;
use bevy_replicon_renet::RenetChannelsExt;
use untypes_core::cli::{CliOptions, NetMode};

/// Unique identifier for this game's protocol version.
/// Clients and servers with different values cannot connect to each other.
const PROTOCOL_ID: u64 = 0x556e_6861_756e_7465; // "Unhaunte" in bytes

/// Maximum simultaneous connections a server will accept.
const MAX_CLIENTS: usize = 4;

pub(super) fn app_setup(app: &mut App) {
    app.add_systems(Startup, startup_transport_system);
}

fn startup_transport_system(
    cli: Res<CliOptions>,
    channels: Res<RepliconChannels>,
    mut commands: Commands,
) {
    let Ok(current_time) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        error!("System clock is before UNIX epoch; cannot initialize network transport.");
        return;
    };

    let connection_config = ConnectionConfig {
        server_channels_config: channels.server_configs(),
        client_channels_config: channels.client_configs(),
        ..Default::default()
    };

    match &cli.net_mode {
        NetMode::Offline => {
            // Singleplayer — no transport needed.
        }
        NetMode::Host { port, .. } => {
            let port = *port;
            // TODO Phase 1.4: switch to Secure with a per-session private key distributed
            // via the Hub's JWT ticket system. Unsecure is intentional here during Phase 1.3.
            let server_config = ServerConfig {
                current_time,
                max_clients: MAX_CLIENTS,
                protocol_id: PROTOCOL_ID,
                public_addresses: vec![SocketAddr::from(([0, 0, 0, 0], port))],
                authentication: ServerAuthentication::Unsecure,
            };
            let socket = match UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], port))) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to bind UDP socket on port {port}: {e}");
                    return;
                }
            };
            let transport = match NetcodeServerTransport::new(server_config, socket) {
                Ok(t) => t,
                Err(e) => {
                    error!("Failed to create netcode server transport: {e}");
                    return;
                }
            };
            commands.insert_resource(RenetServer::new(connection_config));
            commands.insert_resource(transport);
            info!("Replicon transport: listening on UDP port {port}");
        }
        NetMode::Join { address, ticket } => {
            let server_addr: SocketAddr = match address.parse() {
                Ok(a) => a,
                Err(e) => {
                    error!("Invalid server address '{address}': {e}");
                    return;
                }
            };
            // Derive a client_id from the current time to ensure uniqueness across restarts.
            // TODO Phase 1.4: use the installation_id from CliOptions as the stable client_id.
            let client_id = current_time.as_micros() as u64;

            // Encode the JWT ticket into the 256-byte user_data field so the
            // dedicated server can validate the connection before allocating any
            // game state.  The ticket bytes are written starting at offset 0;
            // the remainder is zero-padded.  The server reads until the first
            // null byte, so the JWT must not contain null bytes (it won't, as
            // it is base64url + '.' separated).
            let user_data = ticket.as_deref().map(|t| {
                let mut data = [0u8; bevy_renet::netcode::NETCODE_USER_DATA_BYTES];
                let bytes = t.as_bytes();
                let len = bytes
                    .len()
                    .min(bevy_renet::netcode::NETCODE_USER_DATA_BYTES);
                data[..len].copy_from_slice(&bytes[..len]);
                data
            });

            let authentication = ClientAuthentication::Unsecure {
                protocol_id: PROTOCOL_ID,
                client_id,
                server_addr,
                user_data,
            };
            let socket = match UdpSocket::bind("0.0.0.0:0") {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to bind UDP socket for client: {e}");
                    return;
                }
            };
            let transport = match NetcodeClientTransport::new(current_time, authentication, socket)
            {
                Ok(t) => t,
                Err(e) => {
                    error!("Failed to create netcode client transport: {e}");
                    return;
                }
            };
            commands.insert_resource(RenetClient::new(connection_config));
            commands.insert_resource(transport);
            info!("Replicon transport: connecting to {address} as client_id {client_id}");
        }
    }
}
