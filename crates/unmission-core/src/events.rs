use bevy::prelude::*;

#[derive(Debug, Clone, Message, Default)]
pub struct LevelReadyEvent {
    pub open_van: bool,
}

#[derive(Debug, Clone, Message)]
pub struct MapGeometryInitializedEvent {
    pub map_size: (usize, usize, usize),
    pub origin: (i32, i32, i32),
}

#[derive(Debug, Clone, Message, Default)]
/// Local-only request to leave the current mission UI flow.
///
/// This is not an authority mission-conclusion signal and must not be used to
/// end the mission for other players.
pub struct QuitMissionEvent;
