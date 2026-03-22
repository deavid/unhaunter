use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use unreplicon_core::network_id::NetworkId;

/// Local event fired when a player dies (server-authoritative).
///
/// Written by the health / sanity system when a player's HP reaches zero.
/// Read by death-handling systems (e.g. spectator mode activation).
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct PlayerDiedEvent {
    pub id: NetworkId,
}
