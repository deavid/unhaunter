use std::net::Ipv4Addr;

use crate::resources::TransportConfig;
use bevy::prelude::*;
use bevy_quinnet::client::{QuinnetClient, ClientConnectionConfiguration, certificate::CertificateVerificationMode, connection::ClientAddrConfiguration};
use bevy_quinnet::server::{QuinnetServer, ServerEndpointConfiguration, EndpointAddrConfiguration, certificate::CertificateRetrievalMode};
use bevy_replicon_quinnet::ChannelsConfigurationExt;
use bevy_replicon::prelude::{RepliconChannels, FromClient, ClientId};
use unprofile_core::profile::RuntimeInstallationId;
use unreplicon_core::resources::{AuthorityRole, DisconnectRequest, LobbyPresenceRole};
use unreplicon_core::messages::ConnectionTicketMessage;
use unhub_client::tickets::{ConnectionTicket, encode_ticket};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};

pub(super) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        startup_transport_system.run_if(
            (resource_exists::<RuntimeInstallationId>.or(resource_exists::<AuthorityRole>))
                .and(not(resource_exists::<QuinnetClient>))
                .and(not(resource_exists::<QuinnetServer>))
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
    replicon_channels: Res<RepliconChannels>,
    _installation_id: Option<Res<RuntimeInstallationId>>,
    mut client: ResMut<QuinnetClient>,
    transport_config: Res<TransportConfig>,
) {
    let Some(req) = ev.read().last() else {
        return;
    };

    info!(
        "handle_hub_connection_request: attempting to connect to Hub room at {}",
        req.address
    );

    let skip_ssl = if let TransportConfig::Join { skip_ssl_verification, .. } = *transport_config {
        skip_ssl_verification
    } else {
        false
    };

    let cert_mode = if skip_ssl {
        CertificateVerificationMode::SkipVerification
    } else {
        CertificateVerificationMode::SignedByCertificateAuthority
    };

    let addr_config = ClientAddrConfiguration::from_strings(
        &req.address,
        "0.0.0.0:0"
    ).unwrap_or_else(|e| {
        error!("Failed to parse address {}: {}", req.address, e);
        ClientAddrConfiguration {
            server_addr: "127.0.0.1:0".parse().unwrap(),
            server_hostname: "localhost".to_string(),
            local_bind_addr: "0.0.0.0:0".parse().unwrap(),
        }
    });

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
            info!("Quinnet: opening connection to {}", req.address);
        }
        Err(e) => {
            error!("Failed to open quinnet connection: {e}");
            return;
        }
    }

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
    _installation_id: Option<Res<RuntimeInstallationId>>,
    mut commands: Commands,
    mut server: ResMut<QuinnetServer>,
    mut _client: ResMut<QuinnetClient>,
) {
    info!(
        "startup_transport_system: initializing transport (config={:?})",
        transport_config
    );

    match &*transport_config {
        TransportConfig::Offline => {}
        TransportConfig::PeerHost {
            port,
            bind_addresses: _,
            cert_file,
            key_file,
            skip_ssl_verification,
        } => {
            let cert_mode = if *skip_ssl_verification {
                CertificateRetrievalMode::GenerateSelfSigned { server_hostname: "localhost".to_string() }
            } else {
                match (cert_file, key_file) {
                    (Some(c), Some(k)) => CertificateRetrievalMode::LoadFromFile {
                        cert_file: c.clone(),
                        key_file: k.clone(),
                    },
                    _ => {
                        error!("Server started in PeerHost mode without certificates and skip_ssl_verification is false. Use --skip-ssl-verification to run without SSL (LAN only).");
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
                error!("Failed to start Quinnet server: {e}");
                return;
            }
            info!("Replicon transport: listening on UDP port {port} (Quinnet)");
        }
        TransportConfig::Join { address, ticket: _, skip_ssl_verification } => {
            let cert_mode = if *skip_ssl_verification {
                CertificateVerificationMode::SkipVerification
            } else {
                CertificateVerificationMode::SignedByCertificateAuthority
            };

            let addr_config = ClientAddrConfiguration::from_strings(
                address,
                "0.0.0.0:0"
            ).unwrap_or_else(|e| {
                error!("Failed to parse address {}: {}", address, e);
                ClientAddrConfiguration {
                    server_addr: "127.0.0.1:0".parse().unwrap(),
                    server_hostname: "localhost".to_string(),
                    local_bind_addr: "0.0.0.0:0".parse().unwrap(),
                }
            });

            let config = ClientConnectionConfiguration {
                addr_config,
                cert_mode,
                defaultables: bevy_quinnet::client::ClientConnectionConfigurationDefaultables {
                    send_channels_cfg: replicon_channels.client_configs(),
                    ..Default::default()
                },
            };

            match _client.open_connection(config) {
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
    mut events: MessageReader<bevy_quinnet::client::connection::ConnectionEvent>,
    transport_config: Res<TransportConfig>,
    installation_id: Option<Res<RuntimeInstallationId>>,
    mut writer: MessageWriter<FromClient<ConnectionTicketMessage>>,
) {
    for _ in events.read() {
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

        writer.write(FromClient {
            client_id: ClientId::Server,
            message: ConnectionTicketMessage { ticket: ticket_str },
        });
        info!("Sent ConnectionTicketMessage to server");
    }
}
