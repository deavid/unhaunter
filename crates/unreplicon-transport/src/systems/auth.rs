use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServer;
use bevy_replicon::prelude::*;
use bevy_replicon::shared::backend::connected_client::NetworkId;
use unhub_client::protocol::DedicatedToProcMan;
use unreplicon_core::messages::ConnectionTicketMessage;
use unreplicon_core::ownership::OwnerId;
use unreplicon_core::resources::ClientUuidMap;

use crate::resources::{ProcManChannel, RoomAuth};

#[derive(Component)]
struct AuthTimeout(Timer);

use crate::plugin::NetworkRole;

pub(super) fn app_setup(app: &mut App, _network_role: NetworkRole) {
    app.add_client_event::<ConnectionTicketMessage>(Channel::Ordered);

    app.add_systems(Update, (insert_auth_timeout, enforce_auth_timeout));
    app.add_observer(process_bouncer_handshake);
    app.add_observer(on_client_disconnected_observer);
}

fn insert_auth_timeout(
    mut commands: Commands,
    q_new_clients: Query<
        Entity,
        (
            With<ConnectedClient>,
            Without<AuthorizedClient>,
            Without<AuthTimeout>,
        ),
    >,
) {
    for e in q_new_clients.iter() {
        // FIXME: Bypassing for now  the AUTH to see if at least we can receive SOMETHING.
        commands.entity(e).insert(AuthorizedClient);
        // commands
        //     .entity(e)
        //     .insert(AuthTimeout(Timer::from_seconds(10.0, TimerMode::Once)));
    }
}

fn enforce_auth_timeout(
    mut commands: Commands,
    mut q_timeouts: Query<(Entity, &NetworkId, &mut AuthTimeout), Without<AuthorizedClient>>,
    time: Res<Time>,
    mut server: ResMut<QuinnetServer>,
) {
    for (entity, network_id, mut timeout) in q_timeouts.iter_mut() {
        timeout.0.tick(time.delta());
        if timeout.0.is_finished() {
            error!("Client {:?} auth timeout! Double Tap engaged.", entity);
            if let Some(endpoint) = server.get_endpoint_mut() {
                endpoint.try_disconnect_client(network_id.get());
            }
            commands.entity(entity).despawn();
        }
    }
}

/// System that listens for ConnectionTicketMessage from clients via standard Replicon MessageReader.
fn process_bouncer_handshake(
    trigger: On<FromClient<ConnectionTicketMessage>>,
    protocol_hash: Res<ProtocolHash>,
    room_auth: Res<RoomAuth>,
    procman: Option<Res<ProcManChannel>>,
    mut uuid_map: ResMut<ClientUuidMap>,
    mut commands: Commands,
    mut server: ResMut<QuinnetServer>,
    q_network_ids: Query<&NetworkId>,
) {
    let FromClient { client_id, message } = trigger.event();
    debug!("process_bouncer_handshake: event: {client_id:?}, {message:?}");
    let ClientId::Client(client_entity) = client_id else {
        return;
    };
    let client_entity = *client_entity;

    let ticket_str = &message.ticket;
    let req_hash = &message.protocol_hash;

    let expected_hash = serde_json::to_string(&*protocol_hash).unwrap_or_default();
    if req_hash != &expected_hash {
        error!(
            "Rejecting client {:?}: Protocol version hash mismatch (expected {}, got {})",
            client_entity, expected_hash, req_hash
        );
        if let Ok(network_id) = q_network_ids.get(client_entity)
            && let Some(endpoint) = server.get_endpoint_mut()
        {
            endpoint.try_disconnect_client(network_id.get());
        }
        return;
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
            if let Ok(network_id) = q_network_ids.get(client_entity)
                && let Some(endpoint) = server.get_endpoint_mut()
            {
                endpoint.try_disconnect_client(network_id.get());
            }
            commands.entity(client_entity).despawn();
            return;
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
            if let Ok(network_id) = q_network_ids.get(client_entity)
                && let Some(endpoint) = server.get_endpoint_mut()
            {
                endpoint.try_disconnect_client(network_id.get());
            }
            commands.entity(client_entity).despawn();
            return;
        }
    };

    if ticket.room_code != expected_room {
        error!(
            "Rejecting client {:?} (room mismatch): expected '{}', got '{}'",
            client_entity, expected_room, ticket.room_code
        );
        if let Ok(network_id) = q_network_ids.get(client_entity)
            && let Some(endpoint) = server.get_endpoint_mut()
        {
            endpoint.try_disconnect_client(network_id.get());
        }
        commands.entity(client_entity).despawn();
        return;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if ticket.exp < now {
        error!("Rejecting client {:?}: Ticket expired", client_entity);
        if let Ok(network_id) = q_network_ids.get(client_entity)
            && let Some(endpoint) = server.get_endpoint_mut()
        {
            endpoint.try_disconnect_client(network_id.get());
        }
        commands.entity(client_entity).despawn();
        return;
    }

    let owner_id = OwnerId::Client(client_entity);
    uuid_map.0.insert(owner_id, ticket.player_uuid);
    info!(
        "Client {:?} authenticated successfully for Player: {}",
        client_entity, ticket.player_uuid
    );

    commands
        .entity(client_entity)
        .insert(AuthorizedClient)
        .remove::<AuthTimeout>();

    // Notify procman so it can track player count and extend the room's lifetime.
    if let Some(ref procman) = procman {
        let _ = procman.tx.send(DedicatedToProcMan::PlayerJoined {
            player_uuid: ticket.player_uuid,
        });
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
