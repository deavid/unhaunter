use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServer;
use bevy_replicon::prelude::{FromClient, ConnectedClient};
use bevy_replicon::shared::backend::connected_client::NetworkId;
use unhub_client::protocol::DedicatedToProcMan;
use unreplicon_core::ownership::OwnerId;
use unreplicon_core::resources::ClientUuidMap;
use unreplicon_core::messages::ConnectionTicketMessage;

use crate::resources::{ProcManChannel, RoomAuth};

pub(super) fn app_setup(app: &mut App) {
    app.add_systems(Update, validate_new_connection_ticket);
    app.add_observer(on_client_disconnected_observer);
}

/// System that listens for ConnectionTicketMessage from clients to validate them.
fn validate_new_connection_ticket(
    mut events: MessageReader<FromClient<ConnectionTicketMessage>>,
    mut server: ResMut<QuinnetServer>,
    room_auth: Res<RoomAuth>,
    procman: Option<Res<ProcManChannel>>,
    mut uuid_map: ResMut<ClientUuidMap>,
    q_connected: Query<&NetworkId, With<ConnectedClient>>,
) {
    for req in events.read() {
        let client_id = req.client_id;
        let client_entity = match client_id {
            bevy_replicon::prelude::ClientId::Client(e) => e,
            bevy_replicon::prelude::ClientId::Server => continue, // Should not happen for this message
        };
        let ticket_str = &req.message.ticket;
        let protocol_version = &req.message.protocol_version;

        // Security check: if the entity is not even a connected client anymore, ignore.
        let Ok(network_id) = q_connected.get(client_entity) else {
            warn!(
                "Received ticket for non-existent client entity {:?}",
                client_entity
            );
            continue;
        };

        if protocol_version != "0.4.0-dev" {
            error!(
                "Rejecting client {:?}: Protocol version mismatch (expected '0.4.0-dev', got '{}')",
                client_entity, protocol_version
            );
            if let Some(endpoint) = server.get_endpoint_mut() {
                endpoint.try_disconnect_client(network_id.get());
            }
            continue;
        }

        info!(
            "validate_new_connection_ticket: received ticket for entity {:?} (procman={}, room_assigned={})",
            client_entity,
            procman.is_some(),
            room_auth.room_code.is_some(),
        );

        let (expected_secret, expected_room) = if procman.is_some() {
            let secret = room_auth.ticket_hmac_secret.as_deref().unwrap_or_default();
            let room = room_auth.room_code.as_deref().unwrap_or_default();

            if secret.is_empty() || room.is_empty() {
                error!(
                    "Rejecting client {:?}: Server is idle/unassigned.",
                    client_entity
                );
                if let Some(endpoint) = server.get_endpoint_mut() {
                    endpoint.try_disconnect_client(network_id.get());
                }
                continue;
            }
            (secret, room)
        } else {
            ("", ":DIRECT")
        };

        let ticket = match unhub_client::tickets::decode_ticket_string(ticket_str, expected_secret) {
            Ok(t) => t,
            Err(e) => {
                error!(
                    "Rejecting client {:?}: Ticket validation failed: {}",
                    client_entity, e
                );
                if let Some(endpoint) = server.get_endpoint_mut() {
                    endpoint.try_disconnect_client(network_id.get());
                }
                continue;
            }
        };

        if ticket.room_code != expected_room {
            error!(
                "Rejecting client {:?} (room mismatch): expected '{}', got '{}'",
                client_entity, expected_room, ticket.room_code
            );
            if let Some(endpoint) = server.get_endpoint_mut() {
                endpoint.try_disconnect_client(network_id.get());
            }
            continue;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if ticket.exp < now {
            error!("Rejecting client {:?}: Ticket expired", client_entity);
            if let Some(endpoint) = server.get_endpoint_mut() {
                endpoint.try_disconnect_client(network_id.get());
            }
            continue;
        }

        let owner_id = OwnerId::Client(client_entity);
        uuid_map.0.insert(owner_id, ticket.player_uuid);
        info!(
            "Client {:?} authenticated successfully for Player: {}",
            client_entity, ticket.player_uuid
        );

        // Notify procman so it can track player count and extend the room's lifetime.
        if let Some(procman) = procman.as_ref() {
            let _ = procman.tx.send(DedicatedToProcMan::PlayerJoined {
                player_uuid: ticket.player_uuid,
            });
        }
    }
}

/// Observes each disconnecting client entity to notify procman of the updated player count.
fn on_client_disconnected_observer(
    trigger: On<Remove, bevy_replicon::prelude::ConnectedClient>,
    procman: Option<Res<ProcManChannel>>,
    mut uuid_map: ResMut<ClientUuidMap>,
    q_connected: Query<(), With<bevy_replicon::prelude::ConnectedClient>>,
) {
    let entity = trigger.entity;
    let owner_id = OwnerId::Client(entity);

    // The entity still has ConnectedClient during Remove observers, so subtract 1 for the
    // entity being removed.
    let remaining_count = q_connected.iter().count().saturating_sub(1);

    let player_uuid = uuid_map.0.remove(&owner_id);

    if let Some(procman) = procman.as_ref() {
        match player_uuid {
            Some(uuid) => {
                let _ = procman.tx.send(DedicatedToProcMan::PlayerLeft {
                    player_uuid: uuid,
                    remaining_count,
                });
                info!(
                    "on_client_disconnected_observer: entity {:?} ({}) left; remaining={}",
                    entity, uuid, remaining_count
                );
            }
            None => {
                warn!(
                    "on_client_disconnected_observer: no UUID found for entity {:?}; cannot notify procman",
                    entity
                );
            }
        }
    }
}
