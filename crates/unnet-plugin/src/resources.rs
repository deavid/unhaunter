use bevy::prelude::*;
use std::collections::VecDeque;
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

/// Represents a single remote client connection (used on the host side).
pub(crate) struct ClientConnection {
    pub stream: std::net::TcpStream,
    pub read_buffer: String,
    pub write_queue: VecDeque<NetworkMessage>,
    pub handshake: HandshakeState,
    pub installation_id: Option<uuid::Uuid>,
    pub associated_id: Option<unnet_core::network_id::NetworkId>,
    pub needs_full_sync: bool,
}

#[derive(Resource, Default)]
pub(crate) enum NetworkConn {
    /// No network activity.
    #[default]
    Disconnected,
    /// Host: listening for client connections, managing 0..N active clients.
    Host {
        listeners: Vec<std::net::TcpListener>,
        clients: Vec<ClientConnection>,
    },
    /// Client: single connection to the host.
    Client {
        stream: std::net::TcpStream,
        read_buffer: String,
        write_queue: VecDeque<NetworkMessage>,
        handshake: HandshakeState,
    },
}

impl NetworkConn {
    /// Returns true if this is a Host with at least one client, or a connected Client.
    pub(crate) fn is_active(&self) -> bool {
        match self {
            Self::Host { clients, .. } => !clients.is_empty(),
            Self::Client { .. } => true,
            Self::Disconnected => false,
        }
    }

    /// Send a message on the client's connection to the host. Does nothing if not Client.
    pub(crate) fn client_send(&mut self, msg: NetworkMessage) {
        if let Self::Client { write_queue, .. } = self {
            write_queue.push_back(msg);
        }
    }

    /// Broadcast a message to ALL connected clients. Does nothing if not Host.
    pub(crate) fn host_broadcast(&mut self, msg: NetworkMessage) {
        if let Self::Host { clients, .. } = self {
            for client in clients.iter_mut() {
                client.write_queue.push_back(msg.clone());
            }
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
