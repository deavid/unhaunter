use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// The raw ticket data that will be serialized.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConnectionTicket {
    pub room_code: String,
    pub installation_id: Uuid,
    pub player_uuid: Uuid,
    pub exp: u64,
}

const PAYLOAD_SIZE: usize = 224;
const MAC_SIZE: usize = 32;
pub const TICKET_SIZE: usize = PAYLOAD_SIZE + MAC_SIZE; // Exactly 256 bytes

/// Encodes a ticket into the fixed 256-byte `user_data` buffer expected by Renet.
pub fn encode_ticket(
    ticket: &ConnectionTicket,
    hmac_secret: &str,
) -> anyhow::Result<[u8; TICKET_SIZE]> {
    let mut user_data = [0u8; TICKET_SIZE];

    // 1. Serialize the struct into the first 224 bytes.
    // Postcard returns the sub-slice written. The rest of the 224 bytes remain 0.
    postcard::to_slice(ticket, &mut user_data[..PAYLOAD_SIZE])
        .map_err(|e| anyhow::anyhow!("Failed to serialize ticket: {}", e))?;

    // 2. Compute the HMAC over the entire padded 224-byte block.
    let mut mac = HmacSha256::new_from_slice(hmac_secret.as_bytes())
        .map_err(|e| anyhow::anyhow!("Invalid HMAC key length: {}", e))?;
    mac.update(&user_data[..PAYLOAD_SIZE]);
    let result = mac.finalize().into_bytes();

    // 3. Write the 32-byte MAC to the end of the buffer.
    user_data[PAYLOAD_SIZE..].copy_from_slice(&result);

    Ok(user_data)
}

/// Decodes and verifies a 256-byte ticket buffer.
pub fn decode_ticket(
    user_data: &[u8; TICKET_SIZE],
    hmac_secret: &str,
) -> anyhow::Result<ConnectionTicket> {
    // 1. Verify the HMAC over the first 224 bytes.
    let mut mac = HmacSha256::new_from_slice(hmac_secret.as_bytes())
        .map_err(|e| anyhow::anyhow!("Invalid HMAC key length: {}", e))?;
    mac.update(&user_data[..PAYLOAD_SIZE]);

    // Verify the signature using a constant-time check to prevent timing attacks.
    mac.verify_slice(&user_data[PAYLOAD_SIZE..])
        .map_err(|_| anyhow::anyhow!("Invalid ticket signature"))?;

    // 2. Deserialize the payload. Postcard safely ignores the trailing zeros.
    let ticket: ConnectionTicket = postcard::from_bytes(&user_data[..PAYLOAD_SIZE])
        .map_err(|e| anyhow::anyhow!("Failed to deserialize ticket: {}", e))?;

    Ok(ticket)
}
