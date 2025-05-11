use bevy::prelude::*;

/// Enum to track which mission selection mode we're in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MissionSelectMode {
    /// Campaign missions have predefined difficulty
    Campaign,
    /// Custom missions use a difficulty selected by the player
    Custom,
}

/// Resource to track the current mission selection mode
#[derive(Resource, Debug, Default)]
pub struct CurrentMissionSelectMode(pub MissionSelectMode);

impl Default for MissionSelectMode {
    fn default() -> Self {
        Self::Campaign
    }
}
