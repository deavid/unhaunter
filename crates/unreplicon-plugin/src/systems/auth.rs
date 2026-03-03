use bevy::prelude::*;
use bevy_renet::RenetServer;
use bevy_renet::netcode::NetcodeServerTransport;
use bevy_replicon::prelude::ConnectedClient;
use bevy_replicon::shared::backend::connected_client::NetworkId;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use unreplicon_core::ownership::OwnerId;
use unreplicon_core::resources::ClientUuidMap;
use uuid::Uuid;

use crate::systems::procman::RoomAuth;

/// JWT claims structure — must match what the Hub generates in `tickets.rs`.
#[derive(Debug, Serialize, Deserialize)]
struct TicketClaims {
    room_code: String,
    installation_id: String,
    player_uuid: String,
    exp: u64,
}

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

    // No procman channel → hub-less direct-connect: no tickets exist, accept unconditionally.
    if procman.is_none() {
        info!(
            "Client {:?} connected (hub-less direct-connect; authentication skipped).",
            client_id
        );
        // Fallback: deterministic UUID for development/hub-less
        let uuid = Uuid::from_u128(client_id as u128);
        uuid_map.0.insert(owner_id, uuid);
        return;
    }

    // Reject all connections until a room is assigned and we have a secret.
    let (Some(hmac_secret), Some(room_code)) =
        (&room_auth.ticket_hmac_secret, &room_auth.room_code)
    else {
        // No room assigned yet — this server is idle or resetting.
        warn!(
            "Rejecting client {:?}: no room assigned (server idle).",
            client_id
        );
        server.disconnect(client_id);
        return;
    };

    // Extract user_data from the netcode transport.
    let ticket_opt = transport
        .as_ref()
        .and_then(|t| t.user_data(client_id))
        .and_then(|data| {
            // The JWT was written as raw bytes, null-padded to 256 bytes.
            let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
            std::str::from_utf8(&data[..end]).ok().map(str::to_owned)
        })
        .filter(|s| !s.is_empty());

    let Some(ticket) = ticket_opt else {
        warn!(
            "Rejecting client {:?}: no connection ticket in user_data.",
            client_id
        );
        server.disconnect(client_id);
        return;
    };

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    match decode::<TicketClaims>(
        &ticket,
        &DecodingKey::from_secret(hmac_secret.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => {
            if token_data.claims.room_code != *room_code {
                warn!(
                    "Rejecting client {:?} (room_code mismatch): expected '{}', got '{}'.",
                    client_id, room_code, token_data.claims.room_code
                );
                server.disconnect(client_id);
            } else {
                info!("Client {:?} authenticated successfully.", client_id);
                if let Ok(uuid) = Uuid::parse_str(&token_data.claims.player_uuid) {
                    uuid_map.0.insert(owner_id, uuid);
                }
            }
        }
        Err(e) => {
            warn!(
                "Rejecting client {:?} (JWT decode failed): {}",
                client_id, e
            );
            server.disconnect(client_id);
        }
    }
}
