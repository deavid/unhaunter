use crate::ownership::OwnerId;
use bevy::prelude::*;
use uuid::Uuid;

/// Local intra-app signal emitted by networking systems when a player loses
/// network connectivity during an active mission.
#[derive(Debug, Clone, Message)]
pub struct PlayerNetworkDisconnected {
    pub player_uuid: Uuid,
}

/// Local intra-app signal emitted by networking systems when a previously
/// disconnected player reconnects during an active mission.
#[derive(Debug, Clone, Message)]
pub struct PlayerNetworkReconnected {
    pub player_uuid: Uuid,
    pub new_owner_id: OwnerId,
}
