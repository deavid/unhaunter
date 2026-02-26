use bevy::prelude::*;
use bevy_renet::netcode::NetcodeServerTransport;
use bevy_renet::renet::ServerEvent;
use bevy_renet::{RenetServer, RenetServerEvent};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

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

/// Observes each newly connected client's `user_data`, extracts the JWT ticket,
/// and disconnects any client whose ticket is missing, malformed, or invalid.
///
/// When no [`crate::systems::procman::ProcManChannel`] is present the server is running in
/// hub-less direct-connect mode. In that case all connections are accepted without a ticket
/// because there is no hub to issue tickets and no JWT secret to validate against.
fn validate_new_connection_observer(
    trigger: On<RenetServerEvent>,
    mut server: ResMut<RenetServer>,
    transport: Option<Res<NetcodeServerTransport>>,
    room_auth: Res<RoomAuth>,
    procman: Option<Res<crate::systems::procman::ProcManChannel>>,
) {
    let ServerEvent::ClientConnected { client_id } = &trigger.event().0 else {
        return;
    };
    let client_id = *client_id;

    // No procman channel → hub-less direct-connect: no tickets exist, accept unconditionally.
    if procman.is_none() {
        info!(
            "Client {:?} connected (hub-less direct-connect; authentication skipped).",
            client_id
        );
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

    if !validate_ticket(&ticket, hmac_secret, room_code) {
        warn!(
            "Rejecting client {:?}: connection ticket failed validation.",
            client_id
        );
        server.disconnect(client_id);
    } else {
        info!("Client {:?} authenticated successfully.", client_id);
    }
}

/// Returns `true` iff the JWT is correctly signed, unexpired, and valid for
/// the given room.
fn validate_ticket(ticket: &str, hmac_secret: &str, expected_room_code: &str) -> bool {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    match decode::<TicketClaims>(
        ticket,
        &DecodingKey::from_secret(hmac_secret.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => {
            if token_data.claims.room_code != expected_room_code {
                warn!(
                    "Ticket room_code mismatch: expected '{}', got '{}'.",
                    expected_room_code, token_data.claims.room_code
                );
                false
            } else {
                true
            }
        }
        Err(e) => {
            warn!("JWT decode failed: {}", e);
            false
        }
    }
}
