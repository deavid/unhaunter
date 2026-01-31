use bevy::prelude::*;
use unnet_core::messages::NetworkMessage;

#[derive(Event, Message, Debug)]
pub struct NetworkDataEvent {
    pub message: NetworkMessage,
}
