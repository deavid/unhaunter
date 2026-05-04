use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use bevy::prelude::*;
use bevy_renet2::netcode::NetcodeServerTransport;
use bevy_renet2::prelude::RenetServer;
use bevy_replicon::prelude::{
    AuthorizedClient, Channel, ClientEventAppExt as _, ClientId, ConnectedClient, FromClient,
    ProtocolHash,
};
use bevy_replicon::shared::backend::connected_client::NetworkId;
use unhub_client::protocol::DedicatedToProcMan;
use unreplicon_core::messages::ConnectionTicketMessage;
use unreplicon_core::ownership::OwnerId;
use unreplicon_core::resources::ClientUuidMap;

use crate::resources::{ProcManChannel, RoomAuth};

// --- TOGGLE HERE ---
// true  = Use new Replicon Message handshake (WebTransport compatible)
// false = Use native Renet user_data handshake (UDP only)
const USE_IN_BAND_AUTH: bool = true;

#[derive(Component)]
struct AuthTimeout(Timer);

pub(super) fn app_setup(app: &mut App) {
    app.add_client_event::<ConnectionTicketMessage>(Channel::Ordered);
    app.add_observer(on_client_disconnected_observer);

    if USE_IN_BAND_AUTH {
        app.add_observer(on_client_connected_timeout_observer);
        app.add_systems(Update, enforce_auth_timeout);
        app.add_observer(process_bouncer_handshake);
    } else {
        // Observe Add<ConnectedClient> — fires after bevy_replicon_renet2 has already spawned the
        // client entity with both ConnectedClient and NetworkId, so we can safely retrieve the
        // renet ClientId and map it to a UUID.
        app.add_observer(validate_new_connection_observer);
    }
}

/// Observes each newly connected client entity (after ConnectedClient + NetworkId are
/// already present) to validate the JWT ticket and populate the UUID map.
fn validate_new_connection_observer(
    trigger: On<Add, ConnectedClient>,
    mut server: ResMut<RenetServer>,
    transport: Option<Res<NetcodeServerTransport>>,
    room_auth: Res<RoomAuth>,
    procman: Option<Res<ProcManChannel>>,
    mut uuid_map: ResMut<ClientUuidMap>,
    mut commands: Commands,
    q_network_id: Query<&NetworkId>,
) {
    info!(
        "validate_new_connection_observer: fired for entity {:?} (procman={}, room_assigned={})",
        trigger.entity,
        procman.is_some(),
        room_auth.room_code.is_some()
    );
    let entity = trigger.entity;
    let client_id = match q_network_id.get(entity) {
        Ok(net_id) => net_id.get(),
        Err(_) => {
            warn!("validate_new_connection_observer: no NetworkId on client entity {entity:?}");
            return;
        }
    };
    let owner_id = OwnerId::Client(entity);

    let user_data_bytes = transport
        .as_ref()
        .and_then(|t: &Res<NetcodeServerTransport>| t.user_data(client_id))
        .filter(|data: &[u8; 256]| data.len() == bevy_renet2::netcode::NETCODE_USER_DATA_BYTES);

    let Some(user_data) = user_data_bytes else {
        error!(
            "Rejecting client {:?}: Missing or invalid length user_data.",
            client_id
        );
        server.disconnect(client_id);
        return;
    };

    let (expected_secret, expected_room) = if procman.is_some() {
        let secret = room_auth.ticket_hmac_secret.as_deref().unwrap_or_default();
        let room = room_auth.room_code.as_deref().unwrap_or_default();
        if secret.is_empty() || room.is_empty() {
            server.disconnect(client_id);
            return;
        }
        (secret, room)
    } else {
        ("", ":DIRECT")
    };

    let ticket = match unhub_client::tickets::decode_ticket(&user_data, expected_secret) {
        Ok(t) => t,
        Err(_) => {
            server.disconnect(client_id);
            return;
        }
    };

    if ticket.room_code != expected_room {
        server.disconnect(client_id);
        return;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if ticket.exp < now {
        server.disconnect(client_id);
        return;
    }

    uuid_map.0.insert(owner_id, ticket.player_uuid);

    // NOTE: We manually insert AuthorizedClient here so the rest of your game
    // doesn't break if it expects this new component!
    commands.entity(entity).insert(AuthorizedClient);

    if let Some(procman) = procman.as_ref() {
        let _ = procman.tx.send(DedicatedToProcMan::PlayerJoined {
            player_uuid: ticket.player_uuid,
        });
    }
}

fn on_client_connected_timeout_observer(trigger: On<Add, ConnectedClient>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(AuthTimeout(Timer::from_seconds(10.0, TimerMode::Once)));
}

fn enforce_auth_timeout(
    mut commands: Commands,
    mut q_timeouts: Query<(Entity, &NetworkId, &mut AuthTimeout), Without<AuthorizedClient>>,
    time: Res<Time>,
    mut server: If<ResMut<RenetServer>>,
) {
    for (entity, network_id, mut timeout) in q_timeouts.iter_mut() {
        timeout.0.tick(time.delta());
        if timeout.0.is_finished() {
            error!("Client {:?} auth timeout! Handshake not received.", entity);
            server.disconnect(network_id.get());
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
    mut server: ResMut<RenetServer>,
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

    // ProtocolHash has a private field, we need to compare it somehow.
    // In this repo, it's serialized to JSON to get the value.
    let expected_hash = serde_json::to_string(&*protocol_hash).unwrap_or_default();
    if req_hash != &expected_hash {
        error!(
            "Rejecting client {:?}: Protocol version hash mismatch (expected {}, got {})",
            client_entity, expected_hash, req_hash
        );
        if let Ok(network_id) = q_network_ids.get(client_entity) {
            server.disconnect(network_id.get());
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
            if let Ok(network_id) = q_network_ids.get(client_entity) {
                server.disconnect(network_id.get());
            }
            return;
        }
        (secret, room)
    } else {
        ("", ":DIRECT")
    };
    let decoded_res = B64
        .decode(ticket_str.as_bytes())
        .map_err(|e| anyhow::anyhow!("Base64 fail: {e}"))
        .and_then(|decoded| {
            let mut data = [0u8; unhub_client::tickets::TICKET_SIZE];
            if decoded.len() != data.len() {
                anyhow::bail!("Bad ticket len");
            }
            data.copy_from_slice(&decoded);
            unhub_client::tickets::decode_ticket(&data, expected_secret)
        });

    let ticket = match decoded_res {
        Ok(t) => t,
        Err(e) => {
            error!(
                "Rejecting client {:?}: Ticket validation failed: {}",
                client_entity, e
            );
            if let Ok(network_id) = q_network_ids.get(client_entity) {
                server.disconnect(network_id.get());
            }
            return;
        }
    };

    if ticket.room_code != expected_room {
        error!(
            "Rejecting client {:?} (room mismatch): expected '{}', got '{}'",
            client_entity, expected_room, ticket.room_code
        );
        if let Ok(network_id) = q_network_ids.get(client_entity) {
            server.disconnect(network_id.get());
        }
        return;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if ticket.exp < now {
        error!("Rejecting client {:?}: Ticket expired", client_entity);
        if let Ok(network_id) = q_network_ids.get(client_entity) {
            server.disconnect(network_id.get());
        }
        return;
    }

    let owner_id = OwnerId::Client(client_entity);
    uuid_map.0.insert(owner_id, ticket.player_uuid);
    info!(
        "Client {:?} authenticated successfully for Player: {}",
        client_entity, ticket.player_uuid
    );

    commands.entity(client_entity).insert(AuthorizedClient);

    // Notify procman so it can track player count and extend the room's lifetime.
    if let Some(procman) = procman.as_ref() {
        let _ = procman.tx.send(DedicatedToProcMan::PlayerJoined {
            player_uuid: ticket.player_uuid,
        });
    }
}

/// Observes each disconnecting client entity to notify procman of the updated player count.
fn on_client_disconnected_observer(
    trigger: On<Remove, ConnectedClient>,
    procman: Option<Res<ProcManChannel>>,
    mut uuid_map: ResMut<ClientUuidMap>,
    q_connected: Query<(), With<ConnectedClient>>,
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
