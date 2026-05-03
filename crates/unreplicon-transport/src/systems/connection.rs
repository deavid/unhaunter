use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

use crate::resources::TransportConfig;
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use bevy::prelude::*;
use bevy_quinnet::client::{
    ClientConnectionConfiguration, QuinnetClient, certificate::CertificateVerificationMode,
    client_connected, client_connecting, connection::ClientAddrConfiguration,
};
use bevy_quinnet::server::{
    EndpointAddrConfiguration, QuinnetServer, ServerEndpointConfiguration,
    certificate::CertificateRetrievalMode, server_listening,
};
use bevy_replicon::prelude::*;
use bevy_replicon_quinnet::ChannelsConfigurationExt;
use unhub_client::tickets::{ConnectionTicket, encode_ticket};
use unprofile_core::profile::RuntimeInstallationId;
use unreplicon_core::messages::ConnectionTicketMessage;
use unreplicon_core::resources::{AuthorityRole, DisconnectRequest, LobbyPresenceRole};

pub(super) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        startup_transport_system.run_if(
            (resource_exists::<RuntimeInstallationId>.or(resource_exists::<AuthorityRole>))
                .and(not(server_listening))
                .and(not(client_connecting.or(client_connected)))
                .and(|config: Res<TransportConfig>| !matches!(*config, TransportConfig::Offline)),
        ),
    );
    app.add_systems(Update, monitor_quinnet_client_status);
    app.add_systems(Update, monitor_quinnet_server_clients);
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
    mut client: ResMut<QuinnetClient>,
) {
    if ev.is_empty() {
        return;
    }
    ev.clear();

    warn!(
        "DisconnectRequest received — tearing down client transport and resetting to offline authority"
    );

    client.close_all_connections();

    for entity in q_replicated.iter() {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<LobbyPresenceRole>();
    commands.insert_resource(AuthorityRole);
}

fn handle_hub_connection_request(
    mut ev: MessageReader<unreplicon_core::messages::HubConnectionRequested>,
    mut commands: Commands,
    mut client: ResMut<QuinnetClient>,
    mut transport_config: ResMut<TransportConfig>,
) {
    let Some(req) = ev.read().last() else {
        return;
    };

    info!(
        "handle_hub_connection_request: attempting to connect to Hub room at {}",
        req.address
    );

    // Close any existing connection before opening a new one.
    client.close_all_connections();

    // UPDATE CONFIG ONLY! Do not call client.open_connection here!
    *transport_config = TransportConfig::Join {
        address: req.address.clone(),
        server_hostname: req.server_hostname.clone(),
        ticket: req.ticket.clone(),
        skip_ssl_verification: false,
    };

    // Transition roles: we are now a pure client connected to a dedicated server.
    commands.remove_resource::<AuthorityRole>();
    commands.insert_resource(LobbyPresenceRole);
    info!(
        "handle_hub_connection_request: roles updated — AuthorityRole removed, LobbyPresenceRole inserted"
    );
}

fn startup_transport_system(
    transport_config: Res<TransportConfig>,
    replicon_channels: Res<RepliconChannels>,
    mut commands: Commands,
    mut last_attempt: Local<Option<Instant>>,
    mut server: ResMut<QuinnetServer>,
    mut client: ResMut<QuinnetClient>,
) {
    const RETRY_COOLDOWN: Duration = Duration::from_secs(2);

    let now = Instant::now();
    if let Some(last) = *last_attempt
        && now.duration_since(last) < RETRY_COOLDOWN
    {
        return;
    }
    *last_attempt = Some(now);

    info!(
        "startup_transport_system: initializing transport (config={:?})",
        transport_config
    );

    match &*transport_config {
        TransportConfig::Offline => {}
        TransportConfig::PeerHost {
            port,
            cert_file,
            key_file,
            skip_ssl_verification,
        } => {
            let cert_mode = if *skip_ssl_verification {
                CertificateRetrievalMode::GenerateSelfSigned {
                    server_hostname: "localhost".to_string(),
                }
            } else {
                match (cert_file, key_file) {
                    (Some(c), Some(k)) => {
                        // Pre-flight: check both files are readable before handing them to
                        // bevy_quinnet, so the error message names the exact file and cause
                        // rather than the opaque "Certificate error" string.
                        for (label, path) in [("cert", c.as_str()), ("key", k.as_str())] {
                            if let Err(e) = std::fs::File::open(path) {
                                error!(
                                    "Cannot read {label} file '{}': {e} — check that this \
                                     process has read permission (e.g. `chmod o+r {path}` or \
                                     run as a user with access to Caddy's certificate store)",
                                    path
                                );
                                return;
                            }
                        }
                        CertificateRetrievalMode::LoadFromFile {
                            cert_file: c.clone(),
                            key_file: k.clone(),
                        }
                    }
                    _ => {
                        error!(
                            "Server started in PeerHost mode without certificates and skip_ssl_verification is false. Use --skip-ssl-verification to run with self-signed certs / disable certificate verification for LAN/dev only."
                        );
                        return;
                    }
                }
            };

            let server_config = ServerEndpointConfiguration {
                addr_config: EndpointAddrConfiguration::from_ip(Ipv4Addr::UNSPECIFIED, *port),
                cert_mode,
                defaultables: bevy_quinnet::server::ServerEndpointConfigurationDefaultables {
                    send_channels_cfg: replicon_channels.server_configs(),
                    ..Default::default()
                },
            };

            if let Err(e) = server.start_endpoint(server_config) {
                // Walk the full error source chain — bevy_quinnet wraps the root
                // cause (e.g. "Permission denied") inside "Certificate error",
                // which is opaque on its own.
                use std::error::Error as StdError;
                let mut msg = e.to_string();
                let mut src: Option<&dyn StdError> = e.source();
                while let Some(s) = src {
                    msg.push_str(&format!(": {s}"));
                    src = s.source();
                }
                error!("Failed to start Quinnet server: {msg}");
                return;
            }
            info!("Replicon transport: listening on UDP port {port} (Quinnet)");
        }
        TransportConfig::Join {
            address,
            server_hostname,
            ticket: _,
            skip_ssl_verification,
        } => {
            let cert_mode = if *skip_ssl_verification {
                CertificateVerificationMode::SkipVerification
            } else {
                CertificateVerificationMode::SignedByCertificateAuthority
            };

            let addr_config = match server_hostname {
                Some(hostname) => ClientAddrConfiguration::from_strings_with_name(
                    address,
                    hostname.clone(),
                    "0.0.0.0:0",
                ),
                None => ClientAddrConfiguration::from_strings(address, "0.0.0.0:0"),
            };
            let addr_config = match addr_config {
                Ok(cfg) => cfg,
                Err(e) => {
                    error!("Failed to parse address {}: {}", address, e);
                    return;
                }
            };

            let config = ClientConnectionConfiguration {
                addr_config,
                cert_mode,
                defaultables: bevy_quinnet::client::ClientConnectionConfigurationDefaultables {
                    send_channels_cfg: replicon_channels.client_configs(),
                    ..Default::default()
                },
            };

            match client.open_connection(config) {
                Ok(_) => {
                    info!("Replicon transport: connecting to {address} (Quinnet)");
                }
                Err(e) => {
                    error!("Failed to open Quinnet client connection: {e}");
                    return;
                }
            }

            // Transition roles
            commands.remove_resource::<AuthorityRole>();
            commands.insert_resource(LobbyPresenceRole);
        }
    }
}

fn monitor_quinnet_client_status(client: Res<QuinnetClient>, mut last_state: Local<bool>) {
    let connected = client.is_connected();
    if connected != *last_state {
        if connected {
            info!("QuinnetClient: CONNECTED to server");
        } else {
            warn!("QuinnetClient: NOT CONNECTED to server");
        }
        *last_state = connected;
    }
}

fn monitor_quinnet_server_clients(server: Res<QuinnetServer>, mut last_count: Local<usize>) {
    if let Some(endpoint) = server.get_endpoint() {
        let count = endpoint.clients().len();
        if count != *last_count {
            info!(
                "QuinnetServer: transport-level client count {} → {}",
                *last_count, count
            );
            *last_count = count;
        }
    }
}

fn send_ticket_on_connection(
    transport_config: Res<TransportConfig>,
    installation_id: Option<Res<RuntimeInstallationId>>,
    protocol_hash: Res<ProtocolHash>,
    mut commands: Commands,
    client_state: Res<State<ClientState>>,
    mut ticket_sent: Local<bool>,
    mut throttle: Local<i32>,
    mut req_id: Local<i32>,
) {
    // If we are disconnected, reset the flag so we can send again on next reconnect
    if *client_state.get() != ClientState::Connected {
        *ticket_sent = false;
        return;
    }

    // If we are connected and already sent it, do nothing
    if *ticket_sent {
        return;
    }
    *throttle += 1;
    if *throttle < 5 {
        return;
    }
    *throttle = 0;
    let ticket = match &*transport_config {
        TransportConfig::Join { ticket, .. } => ticket.clone(),
        _ => None,
    };

    let ticket_str = if let Some(t) = ticket {
        t
    } else {
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

    let Ok(hash_val) = serde_json::to_string(&*protocol_hash) else {
        error!("Failed to serialize server protocol hash");
        return;
    };
    *req_id += 1;
    commands.client_trigger(ConnectionTicketMessage {
        id: *req_id,
        ticket: ticket_str,
        protocol_hash: hash_val,
    });

    // *ticket_sent = true;
    info!("Sent ConnectionTicketMessage via client_trigger (Observer Event)");
}
