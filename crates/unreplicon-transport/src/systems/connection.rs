use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use crate::resources::TransportConfig;
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use bevy::prelude::*;
use bevy_renet2::netcode::{
    ClientAuthentication, NativeSocket, NetcodeClientTransport, NetcodeServerTransport,
    ServerAuthentication, ServerSetupConfig,
};
#[cfg(target_arch = "wasm32")]
use bevy_renet2::netcode::{WebTransportClient, WebTransportClientConfig};
use bevy_renet2::prelude::{RenetClient, RenetServer};
use bevy_replicon::prelude::{ClientTriggerExt, ProtocolHash, RepliconChannels};
use bevy_replicon_renet2::RenetChannelsExt;
use bevy_replicon_renet2::renet2::ConnectionConfig;
#[cfg(not(target_arch = "wasm32"))]
use renet2_netcode::{WebTransportServer, WebTransportServerConfig};
use unhub_client::tickets::{ConnectionTicket, encode_ticket};
use unprofile_core::profile::RuntimeInstallationId;
use unreplicon_core::messages::ConnectionTicketMessage;
use unreplicon_core::resources::{AuthorityRole, DisconnectRequest, LobbyPresenceRole};

/// Unique identifier for this game's protocol version.
/// Clients and servers with different values cannot connect to each other.
const PROTOCOL_ID: u64 = 0x556e_6861_756e_7465;

/// Maximum simultaneous connections a server will accept.
const MAX_CLIENTS: usize = 4;

/// Constant to switch between UDP and WebTransport for testing.
/// Set to `true` to use WebTransport, `false` for UDP.
const USE_WEB_TRANSPORT: bool = false;

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
    app.add_systems(Update, send_ticket_on_connection);
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
    time: Res<Time>,
) {
    let Some(req) = ev.read().last() else {
        return;
    };

    info!(
        "handle_hub_connection_request: attempting to connect to Hub room at {}",
        req.address
    );

    let current_time = time.elapsed();

    let connection_config =
        ConnectionConfig::from_channels(channels.server_configs(), channels.client_configs());

    let server_addr: SocketAddr = match req.address.to_socket_addrs() {
        Ok(mut addrs) => match addrs.find(|a| a.is_ipv4()) {
            Some(a) => a,
            None => {
                error!(
                    "DNS resolution returned no IPv4 addresses for '{}'",
                    req.address
                );
                return;
            }
        },
        Err(e) => {
            error!("Invalid server address '{}': {e}", req.address);
            return;
        }
    };

    let installation_id =
        installation_id.expect("RuntimeInstallationId must exist for clients joining via Hub");
    let client_id = installation_id.0.as_u128() as u64;

    let user_data = if let Some(t_str) = req.ticket.as_ref() {
        let mut data = [0u8; bevy_renet2::netcode::NETCODE_USER_DATA_BYTES];
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
        warn!("Hub seems to have sent no ticket and we are crating a default one");
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
        socket_id: 0,
    };

    let transport = if USE_WEB_TRANSPORT {
        #[cfg(target_arch = "wasm32")]
        {
            // FIXME: Simple address parsing for now. WebTransport uses URLs.
            // TODO: Use real SSL certificates and fingerprint validation in the future.
            let server_url = format!("https://{}", req.address);
            let config = WebTransportClientConfig::new(
                url::Url::parse(&server_url).expect("Failed to parse WebTransport URL"),
            );
            let socket = WebTransportClient::new(config);
            NetcodeClientTransport::new(current_time, authentication, socket)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            error!("WebTransport client is only supported on WASM in this implementation.");
            return;
        }
    } else {
        let socket = match UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to bind UDP socket for client: {e}");
                return;
            }
        };
        NetcodeClientTransport::new(
            current_time,
            authentication,
            NativeSocket::new(socket).unwrap(),
        )
    };

    let transport = match transport {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to create netcode client transport: {e}");
            return;
        }
    };

    commands.insert_resource(RenetClient::new(connection_config, false));
    commands.insert_resource(transport);
    commands.insert_resource(TransportConfig::Join {
        address: req.address.clone(),
        ticket: req.ticket.clone(),
    });
    // Transition roles: we are now a pure client connected to a dedicated server.
    commands.remove_resource::<AuthorityRole>();
    commands.insert_resource(LobbyPresenceRole);
    info!(
        "handle_hub_connection_request: roles updated — AuthorityRole removed, LobbyPresenceRole inserted"
    );
}

fn startup_transport_system(
    transport_config: Res<TransportConfig>,
    channels: Res<RepliconChannels>,
    installation_id: Option<Res<RuntimeInstallationId>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    info!(
        "startup_transport_system: initializing transport (config={:?})",
        transport_config
    );
    let current_time = time.elapsed();

    let connection_config =
        ConnectionConfig::from_channels(channels.server_configs(), channels.client_configs());

    match &*transport_config {
        TransportConfig::Offline => {}
        TransportConfig::PeerHost {
            port,
            bind_addresses,
        } => {
            let port = *port;

            let mut public_addresses = vec![
                SocketAddr::from(([0, 0, 0, 0], port)),
                SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], port)),
            ];

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

            let server_config = ServerSetupConfig {
                current_time,
                max_clients: MAX_CLIENTS,
                protocol_id: PROTOCOL_ID,
                socket_addresses: vec![public_addresses],
                authentication: ServerAuthentication::Unsecure,
            };
            let socket = match UdpSocket::bind(("[::]", port))
                .or_else(|_| UdpSocket::bind(("0.0.0.0", port)))
            {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to bind UDP socket on port {port}: {e}");
                    return;
                }
            };
            let transport = if USE_WEB_TRANSPORT {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    // FIXME: Simple self-signed cert for now.
                    // TODO: Load real SSL certificates from filesystem (e.g. Caddy/Let'sEncrypt).
                    // FIXME: IPv4 only — original UDP code bound to [::] for dual-stack support.
                    let (config, _hash) = WebTransportServerConfig::new_selfsigned(
                        SocketAddr::from(([0, 0, 0, 0], port)),
                        MAX_CLIENTS,
                    )
                    .expect("Failed to create self-signed WebTransport config");
                    // FIXME: _hash is discarded — clients need the certificate fingerprint
                    // to trust the self-signed cert. Distribute it out-of-band or switch
                    // to CA-signed certs (Caddy/Let'sEncrypt).
                    let socket =
                        WebTransportServer::new(config, tokio::runtime::Handle::current()).unwrap();
                    NetcodeServerTransport::new(server_config, socket)
                }
                #[cfg(target_arch = "wasm32")]
                {
                    error!("WebTransport server is not supported on WASM.");
                    return;
                }
            } else {
                NetcodeServerTransport::new(server_config, NativeSocket::new(socket).unwrap())
            };

            let transport = match transport {
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
            let server_addr: SocketAddr = match address.to_socket_addrs() {
                Ok(mut addrs) => match addrs.find(|a| a.is_ipv4()) {
                    Some(a) => a,
                    None => {
                        error!("DNS resolution returned no IPv4 addresses for '{address}'");
                        return;
                    }
                },
                Err(e) => {
                    error!("Invalid server address '{address}': {e}");
                    return;
                }
            };

            let installation_id = installation_id
                .expect("RuntimeInstallationId must exist for non-dedicated clients");
            let client_id = installation_id.0.as_u128() as u64;

            let user_data = if let Some(t_str) = ticket {
                let mut data = [0u8; bevy_renet2::netcode::NETCODE_USER_DATA_BYTES];
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
                socket_id: 0,
            };

            let transport = if USE_WEB_TRANSPORT {
                #[cfg(target_arch = "wasm32")]
                {
                    // FIXME: Simple address parsing for now. WebTransport uses URLs.
                    // TODO: Use real SSL certificates and fingerprint validation in the future.
                    let server_url = format!("https://{}", address);
                    let config = WebTransportClientConfig::new(
                        url::Url::parse(&server_url).expect("Failed to parse WebTransport URL"),
                    );
                    let socket = WebTransportClient::new(config);
                    NetcodeClientTransport::new(current_time, authentication, socket)
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    error!("WebTransport client is only supported on WASM in this implementation.");
                    return;
                }
            } else {
                let socket = match UdpSocket::bind("0.0.0.0:0") {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to bind UDP socket for client: {e}");
                        return;
                    }
                };
                NetcodeClientTransport::new(
                    current_time,
                    authentication,
                    NativeSocket::new(socket).unwrap(),
                )
            };

            let transport = match transport {
                Ok(t) => t,
                Err(e) => {
                    error!("Failed to create netcode client transport: {e}");
                    return;
                }
            };
            commands.insert_resource(RenetClient::new(connection_config, false));
            commands.insert_resource(transport);
            info!("Replicon transport: connecting to {address} as client_id {client_id}");
        }
    }
}

fn monitor_renet_client_status(client: Option<Res<RenetClient>>, mut last_state: Local<u8>) {
    let state: u8 = match client {
        None => 0,
        Some(ref client) => {
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

fn send_ticket_on_connection(
    transport_config: Res<TransportConfig>,
    installation_id: Option<Res<RuntimeInstallationId>>,
    protocol_hash: Option<Res<ProtocolHash>>,
    client: Option<Res<RenetClient>>,
    mut ticket_sent: Local<bool>,
    mut commands: Commands,
) {
    let Some(client) = client else {
        *ticket_sent = false;
        return;
    };

    if !client.is_connected() {
        *ticket_sent = false;
        return;
    }

    if *ticket_sent {
        return;
    }

    let Some(protocol_hash) = protocol_hash else {
        return;
    };

    let ticket = match &*transport_config {
        TransportConfig::Join { ticket, .. } => ticket.clone(),
        _ => None,
    };

    let ticket_str = if let Some(t) = ticket {
        t
    } else {
        warn!(
            "Trying to Join but no ticket was found - one generic will be created but this is technically not correct"
        );
        let id = installation_id.as_ref().map(|i| i.0).unwrap_or_default();
        let ticket = ConnectionTicket {
            room_code: ":DIRECT".to_string(),
            installation_id: id,
            player_uuid: id,
            exp: u64::MAX,
        };
        let bytes = encode_ticket(&ticket, "").unwrap_or([0u8; 256]);
        B64.encode(bytes)
    };

    // ProtocolHash has a private field, serialize to get the value
    let Ok(hash_val) = serde_json::to_string(&*protocol_hash) else {
        error!("Failed to serialize server protocol hash");
        return;
    };
    commands.client_trigger(ConnectionTicketMessage {
        ticket: ticket_str,
        protocol_hash: hash_val,
    });

    *ticket_sent = true;
    info!("Sent ConnectionTicketMessage");
}
