use bevy::prelude::*;
use std::collections::VecDeque;
use std::net::TcpStream;
use unnet_core::messages::NetworkMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeState {
    None,
    HelloSent,
    HelloReceived,
    Completed,
}

#[derive(Resource, Default, Debug)]
pub struct PendingMapLoad {
    pub map_filepath: Option<String>,
}

#[derive(Resource, Default)]
pub enum NetworkConn {
    #[default]
    Disconnected,
    Listening(std::net::TcpListener),
    Active {
        stream: TcpStream,
        read_buffer: String,
        write_queue: VecDeque<NetworkMessage>,
        handshake: HandshakeState,
    },
}

impl NetworkConn {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active { .. })
    }

    pub fn send(&mut self, msg: NetworkMessage) {
        if let Self::Active { write_queue, .. } = self {
            write_queue.push_back(msg);
        }
    }
}
