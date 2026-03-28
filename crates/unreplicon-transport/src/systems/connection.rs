use std::net::{SocketAddr, UdpSocket};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::resources::TransportConfig;
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use bevy::prelude::*;
use bevy_renet::netcode::{
    ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication,
    ServerConfig,
};
use bevy_renet::renet::ConnectionConfig;
use bevy_renet::{RenetClient, RenetServer};
use bevy_replicon::prelude::RepliconChannels;
use bevy_replicon_renet::RenetChannelsExt;
use unhub_client::tickets::{ConnectionTicket, encode_ticket};
use unprofile_core::profile::RuntimeInstallationId;
use unreplicon_core::resources::{AuthorityRole, DisconnectRequest, LobbyPresenceRole};

/// Unique identifier for this game's protocol version.
/// Clients and servers with different values cannot connect to each other.
const PROTOCOL_ID: u64 = 0x556e_6861_756e_7465;

/// Maximum simultaneous connections a server will accept.
const MAX_CLIENTS: usize = 4;

pub(super) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        startup_transport_system.run_if(
            (resource_exists::<RuntimeInstallationId>.or(resource_exists::<AuthorityRole>))
                .and(not(resource_exists::<NetcodeClientTransport>))
                .and(not(resource_exists::<NetcodeServerTransport>))
                .and(|config: Res<TransportConfig>| !matches!(*config, TransportConfig::Offline)),
        ),
    );
    app.add_systems(Update, monitor_renet_client_status);
    app.add_systems(Update, monitor_renet_server_clients);
    app.add_systems(
        Update,
        handle_disconnect_request.run_if(resource_exists::<LobbyPresenceRole>),
    );
    app.add_systems(Update, handle_hub_connection_request);
}

fn handle_disconnect_request(
    mut ev: MessageReader<DisconnectRequest>,
    mut commands: Commands,
    q_replicated: Query<Entity, With<bevy_replicon::prelude::Replicated>>,
) {
    if ev.is_empty() {
        return;
    }
    ev.clear();

    warn!(
        "DisconnectRequest received — tearing down client transport and resetting to offline authority"
    );

    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<NetcodeClientTransport>();

    for entity in q_replicated.iter() {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<LobbyPresenceRole>();
    commands.insert_resource(AuthorityRole);
}

fn handle_hub_connection_request(
    mut ev: MessageReader<unreplicon_core::messages::HubConnectionRequested>,
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    installation_id: Option<Res<RuntimeInstallationId>>,
) {
    let Some(req) = ev.read().next() else {
        return;
    };

    info!(
        "handle_hub_connection_request: attempting to connect to Hub room at {}",
        req.address
    );

    let Ok(current_time) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        error!("System clock is before UNIX epoch; cannot initialize network transport.");
        return;
    };

    let connection_config = ConnectionConfig {
        server_channels_config: channels.server_configs(),
        client_channels_config: channels.client_configs(),
        ..Default::default()
    };

    let server_addr: SocketAddr = match req.address.parse() {
        Ok(a) => a,
        Err(e) => {
            error!("Invalid server address '{}': {e}", req.address);
            return;
        }
    };

    let installation_id =
        installation_id.expect("RuntimeInstallationId must exist for clients joining via Hub");
    let client_id = installation_id.0.as_u128() as u64;

    let user_data = if let Some(t_str) = req.ticket.as_ref() {
        let mut data = [0u8; bevy_renet::netcode::NETCODE_USER_DATA_BYTES];
        if let Ok(decoded) = B64.decode(t_str.as_bytes()) {
            if decoded.len() == data.len() {
                data.copy_from_slice(&decoded);
                Some(data)
            } else {
                error!("Received ticket of invalid length");
                None
            }
        } else {
            error!("Failed to parse Base64 ticket");
            None
        }
    } else {
        let id = installation_id.0;
        let ticket = ConnectionTicket {
            room_code: ":DIRECT".to_string(),
            installation_id: id,
            player_uuid: id,
            exp: u64::MAX,
        };
        Some(encode_ticket(&ticket, "").unwrap_or([0u8; 256]))
    };

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
    let transport = match NetcodeClientTransport::new(current_time, authentication, socket) {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to create netcode client transport: {e}");
            return;
        }
    };

    commands.insert_resource(RenetClient::new(connection_config));
    commands.insert_resource(transport);
}

fn startup_transport_system(
    transport_config: Res<TransportConfig>,
    channels: Res<RepliconChannels>,
    installation_id: Option<Res<RuntimeInstallationId>>,
    mut commands: Commands,
) {
    info!(
        "startup_transport_system: initializing transport (config={:?})",
        transport_config
    );
    let Ok(current_time) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        error!("System clock is before UNIX epoch; cannot initialize network transport.");
        return;
    };

    let connection_config = ConnectionConfig {
        server_channels_config: channels.server_configs(),
        client_channels_config: channels.client_configs(),
        ..Default::default()
    };

    match &*transport_config {
        TransportConfig::Offline => {}
        TransportConfig::PeerHost {
            port,
            bind_addresses,
        } => {
            let port = *port;

            let mut public_addresses = vec![SocketAddr::from(([0, 0, 0, 0], port))];

            for addr_str in bind_addresses {
                match addr_str.parse::<SocketAddr>() {
                    Ok(addr) => public_addresses.push(addr),
                    Err(_) => {
                        if let Ok(ip) = addr_str.parse::<std::net::IpAddr>() {
                            public_addresses.push(SocketAddr::new(ip, port));
                        } else {
                            warn!("Invalid bind address: {}", addr_str);
                        }
                    }
                }
            }

            let server_config = ServerConfig {
                current_time,
                max_clients: MAX_CLIENTS,
                protocol_id: PROTOCOL_ID,
                public_addresses,
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
        TransportConfig::Join { address, ticket } => {
            let server_addr: SocketAddr = match address.parse() {
                Ok(a) => a,
                Err(e) => {
                    error!("Invalid server address '{address}': {e}");
                    return;
                }
            };

            let installation_id = installation_id
                .expect("RuntimeInstallationId must exist for non-dedicated clients");
            let client_id = installation_id.0.as_u128() as u64;

            let user_data = if let Some(t_str) = ticket {
                let mut data = [0u8; bevy_renet::netcode::NETCODE_USER_DATA_BYTES];
                if let Ok(decoded) = B64.decode(t_str.as_bytes()) {
                    if decoded.len() == data.len() {
                        data.copy_from_slice(&decoded);
                        Some(data)
                    } else {
                        error!("Received ticket of invalid length");
                        None
                    }
                } else {
                    error!("Failed to parse Base64 ticket");
                    None
                }
            } else {
                let id = installation_id.0;
                let ticket = ConnectionTicket {
                    room_code: ":DIRECT".to_string(),
                    installation_id: id,
                    player_uuid: id,
                    exp: u64::MAX,
                };
                Some(encode_ticket(&ticket, "").unwrap_or([0u8; 256]))
            };

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

fn monitor_renet_client_status(client: Option<Res<RenetClient>>, mut last_state: Local<u8>) {
    let state: u8 = match client.as_ref() {
        None => 0,
        Some(client) => {
            if client.is_connected() {
                2
            } else if client.is_disconnected() {
                3
            } else {
                1
            }
        }
    };
    if state != *last_state {
        match state {
            0 => debug!("RenetClient: resource absent (was state {})", *last_state),
            1 => info!("RenetClient: connecting to server..."),
            2 => info!("RenetClient: CONNECTED to server"),
            3 => {
                let reason = client
                    .as_ref()
                    .and_then(|client| client.disconnect_reason());
                warn!(
                    "RenetClient: DISCONNECTED from server (reason: {:?})",
                    reason
                );
            }
            _ => {}
        }
        *last_state = state;
    }
}

fn monitor_renet_server_clients(server: Option<Res<RenetServer>>, mut last_count: Local<usize>) {
    let Some(server) = server else {
        return;
    };
    let count = server.clients_id().len();
    if count != *last_count {
        if count > *last_count {
            info!(
                "RenetServer: transport-level client count {} → {} (client joined at transport layer)",
                *last_count, count
            );
        } else {
            info!(
                "RenetServer: transport-level client count {} → {} (client left at transport layer)",
                *last_count, count
            );
        }
        *last_count = count;
    }
}
