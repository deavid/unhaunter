use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Claims embedded in a per-room connection ticket.
///
/// The Hub signs this with the procman's HMAC-SHA256 key.  The dedicated game
/// server receives the same key via the `AssignRoom` stdin message and uses it
/// to validate incoming connections.
#[derive(Debug, Serialize, Deserialize)]
pub struct TicketClaims {
    /// The room code the ticket is valid for.
    pub room_code: String,
    /// UUID of the procman instance hosting the room.
    pub installation_id: String,
    /// UUID of the player this ticket was issued for.
    pub player_uuid: String,
    /// Expiry timestamp (Unix seconds, standard JWT `exp` claim).
    pub exp: u64,
}

/// Generates a signed JWT ticket for a player to connect to a specific room.
///
/// The ticket is valid for 5 minutes.  The HMAC key should be the one received
/// from the procman during its handshake.
pub fn generate_ticket(
    hmac_secret: &str,
    room_code: &str,
    installation_id: Uuid,
    player_uuid: Uuid,
) -> anyhow::Result<String> {
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        + 300; // 5 minutes

    let claims = TicketClaims {
        room_code: room_code.to_string(),
        installation_id: installation_id.to_string(),
        player_uuid: player_uuid.to_string(),
        exp,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(hmac_secret.as_bytes()),
    )?;

    Ok(token)
}
