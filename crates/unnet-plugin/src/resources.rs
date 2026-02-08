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
}

#[derive(Resource, Default)]
pub(crate) enum NetworkConn {
    #[default]
    Disconnected,
    Listening(std::net::TcpListener),
    Active {
        stream: TcpStream,
        read_buffer: String,
        write_queue: VecDeque<NetworkMessage>,
        handshake: HandshakeState,
        associated_id: Option<unnet_core::network_id::NetworkId>,
        needs_full_sync: bool,
        host_listener: Option<std::net::TcpListener>,
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
