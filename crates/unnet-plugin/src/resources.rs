use bevy::prelude::*;
use std::collections::VecDeque;
use std::net::TcpStream;
use unnet_core::messages::NetworkMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HandshakeState {
    None,
    HelloSent,
    Completed,
}

#[derive(Resource, Default, Debug)]
pub(crate) struct PendingMapLoad {
    pub map_filepath: Option<String>,
    /// Set to true when the map load is triggered; consumed by
    /// `client_send_input_system` to send a `RequestFullSync` once in `InGame`.
    pub needs_full_sync_request: bool,
}

#[derive(Resource, Default)]
pub(crate) enum NetworkConn {
    #[default]
    Disconnected,
    Listening(Vec<std::net::TcpListener>),
    Active {
        stream: TcpStream,
        read_buffer: String,
        write_queue: VecDeque<NetworkMessage>,
        handshake: HandshakeState,
        installation_id: Option<uuid::Uuid>,
        associated_id: Option<unnet_core::network_id::NetworkId>,
        needs_full_sync: bool,
        host_listeners: Vec<std::net::TcpListener>,
    },
}

impl NetworkConn {
    pub(crate) fn is_active(&self) -> bool {
        matches!(self, Self::Active { .. })
    }

    pub(crate) fn send(&mut self, msg: NetworkMessage) {
        if let Self::Active { write_queue, .. } = self {
            write_queue.push_back(msg);
        }
    }
}

#[derive(Resource, Debug)]
pub(crate) struct PlayerRegistry {
    /// Maps installation_id (UUID) -> assigned NetworkId
    pub uuid_to_network_id: std::collections::HashMap<uuid::Uuid, unnet_core::network_id::NetworkId>,
    /// The next NetworkId to assign to a new player
    pub next_id: u64,
}

impl Default for PlayerRegistry {
    fn default() -> Self {
        Self {
            uuid_to_network_id: std::collections::HashMap::new(),
            next_id: 2,
        }
    }
}

impl PlayerRegistry {
    pub(crate) fn get_or_assign(&mut self, uuid: uuid::Uuid) -> unnet_core::network_id::NetworkId {
        if let Some(&id) = self.uuid_to_network_id.get(&uuid) {
            return id;
        }
        let id = unnet_core::network_id::NetworkId(self.next_id);
        self.next_id += 1;
        self.uuid_to_network_id.insert(uuid, id);
        id
    }
}
