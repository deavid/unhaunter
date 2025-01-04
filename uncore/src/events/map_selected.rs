use bevy::prelude::*;

#[derive(Debug, Clone, Event)]
pub struct MapSelectedEvent {
    pub map_idx: usize,
}
