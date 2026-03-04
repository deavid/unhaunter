use crate::systems::procman::RoomAuth;
use bevy::prelude::*;
use bevy_renet::RenetServer;
use bevy_renet::netcode::NetcodeServerTransport;
use bevy_replicon::prelude::ConnectedClient;
use bevy_replicon::shared::backend::connected_client::NetworkId;
use unreplicon_core::ownership::OwnerId;
use unreplicon_core::resources::ClientUuidMap;

pub(super) fn app_setup(app: &mut App) {
    // Observe Add<ConnectedClient> — fires after bevy_replicon_renet has already spawned the
    // client entity with both ConnectedClient and NetworkId, so we can safely retrieve the
    // renet ClientId and map it to a UUID.
    app.add_observer(validate_new_connection_observer);
}

/// Observes each newly connected client entity (after ConnectedClient + NetworkId are
/// already present) to validate the JWT ticket and populate the UUID map.
fn validate_new_connection_observer(
    trigger: On<Add, ConnectedClient>,
    mut server: ResMut<RenetServer>,
    transport: Option<Res<NetcodeServerTransport>>,
    room_auth: Res<RoomAuth>,
    procman: Option<Res<crate::systems::procman::ProcManChannel>>,
    mut uuid_map: ResMut<ClientUuidMap>,
    q_network_id: Query<&NetworkId>,
) {
    info!(
        "validate_new_connection_observer: fired for entity {:?} (procman={}, room_assigned={})",
        trigger.entity,
        procman.is_some(),
        room_auth.room_code.is_some(),
    );
    let entity = trigger.entity;
    let client_id = match q_network_id.get(entity) {
        Ok(net_id) => net_id.get(),
        Err(_) => {
            warn!(
                "validate_new_connection_observer: no NetworkId on client entity {:?}",
                entity
            );
            return;
        }
    };
    let owner_id = OwnerId::Client(entity);

    // --- New Logic Starts Here ---

    // Extract the raw 256 bytes from Renet's transport
    let user_data_bytes = transport
        .as_ref()
        .and_then(|t| t.user_data(client_id))
        .filter(|data| data.len() == bevy_renet::netcode::NETCODE_USER_DATA_BYTES);

    let Some(user_data) = user_data_bytes else {
        error!(
            "Rejecting client {:?}: Missing or invalid length user_data.",
            client_id
        );
        server.disconnect(client_id);
        return;
    };

    // Determine the expected secret and room based on topology
    let (expected_secret, expected_room) = if procman.is_some() {
        // Hub Mode (Dedicated Server)
        let secret = room_auth.ticket_hmac_secret.as_deref().unwrap_or_default();
        let room = room_auth.room_code.as_deref().unwrap_or_default();

        if secret.is_empty() || room.is_empty() {
            error!(
                "Rejecting client {:?}: Server is idle/unassigned.",
                client_id
            );
            server.disconnect(client_id);
            return;
        }
        (secret, room)
    } else {
        // Direct Connect Mode (PeerHost)
        ("", ":DIRECT")
    };

    // Attempt to decode and verify the HMAC signature
    let ticket = match unhub_client::tickets::decode_ticket(&user_data, expected_secret) {
        Ok(t) => t,
        Err(e) => {
            error!(
                "Rejecting client {:?}: Ticket validation failed: {}",
                client_id, e
            );
            server.disconnect(client_id);
            return;
        }
    };

    // Validate Room Code
    if ticket.room_code != expected_room {
        error!(
            "Rejecting client {:?} (room mismatch): expected '{}', got '{}'",
            client_id, expected_room, ticket.room_code
        );
        server.disconnect(client_id);
        return;
    }

    // Validate Expiration
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if ticket.exp < now {
        error!("Rejecting client {:?}: Ticket expired", client_id);
        server.disconnect(client_id);
        return;
    }

    // Validation passed! Map the UUID to the connection.
    uuid_map.0.insert(owner_id, ticket.player_uuid);
    info!(
        "Client {:?} authenticated successfully for Player: {}",
        client_id, ticket.player_uuid
    );
}
