use bevy::prelude::*;

#[derive(Debug, Clone, Message)]
pub struct MapSelectedEvent {
    pub map_idx: usize,
}
