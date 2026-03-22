use bevy::prelude::*;

use crate::summary::SummaryData;

#[derive(Debug, Clone, Message, Default)]
pub struct LevelReadyEvent {
    pub open_van: bool,
}

#[derive(Debug, Clone, Message)]
pub struct MapGeometryInitializedEvent {
    pub map_size: (usize, usize, usize),
    pub origin: (i32, i32, i32),
}

#[derive(Debug, Clone, Message)]
pub struct MissionCompletedEvent {
    pub summary: SummaryData,
}
