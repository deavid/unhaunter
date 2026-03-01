use bevy::prelude::*;
use bevy_renet::netcode::NetcodeServerTransport;
use bevy_renet::renet::ServerEvent;
use bevy_renet::{RenetServer, RenetServerEvent};
use bevy_replicon::prelude::ClientId;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
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
    // bevy_renet 4.0 fires connection events via `commands.trigger(RenetServerEvent(...))`.
    // Use a Bevy observer to react to each connection individually.
    app.add_observer(validate_new_connection_observer);
}

/// Helper to convert renet ClientId to replicon ClientId.
///
/// In bevy_replicon 0.38, for the Renet backend, the ClientId variant is
/// `Client(Entity)`. We must find the Entity associated with the renet ID.
fn renet_to_replicon(
    renet_id: renet::ClientId,
    q_network_id: &Query<(
        Entity,
        &bevy_replicon::shared::backend::connected_client::NetworkId,
    )>,
) -> Option<ClientId> {
    for (entity, net_id) in q_network_id.iter() {
        if net_id.get() == renet_id {
            return Some(ClientId::Client(entity));
        }
    }
    None
}

/// Observes each newly connected client's `user_data`, extracts the JWT ticket,
/// and disconnects any client whose ticket is missing, malformed, or invalid.
fn validate_new_connection_observer(
    trigger: On<RenetServerEvent>,
    mut server: ResMut<RenetServer>,
    transport: Option<Res<NetcodeServerTransport>>,
    room_auth: Res<RoomAuth>,
    procman: Option<Res<crate::systems::procman::ProcManChannel>>,
    mut uuid_map: ResMut<ClientUuidMap>,
    q_network_id: Query<(
        Entity,
        &bevy_replicon::shared::backend::connected_client::NetworkId,
    )>,
) {
    let ServerEvent::ClientConnected { client_id } = &trigger.event().0 else {
        return;
    };
    let client_id = *client_id;
    let replicon_client_id =
        renet_to_replicon(client_id, &q_network_id).unwrap_or(ClientId::Server);

    // No procman channel → hub-less direct-connect: no tickets exist, accept unconditionally.
    if procman.is_none() {
        info!(
            "Client {:?} connected (hub-less direct-connect; authentication skipped).",
            client_id
        );
        // Fallback: deterministic UUID for development/hub-less
        let uuid = Uuid::from_u128(client_id as u128);
        uuid_map.0.insert(replicon_client_id, uuid);
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
                    uuid_map.0.insert(replicon_client_id, uuid);
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
