use crate::generated::base1::Base1Concept;
use bevy::prelude::Event;
use unwalkie_types::VoiceLineData;

/// Sending this event will cause the walkie to play a message.
#[derive(Clone, Debug, Event, PartialEq, Eq, Hash)]
pub enum WalkieEvent {
    /// When the player forgets the stuff in the van.
    GearInVan,
    /// When the Ghost rage is near its limit.
    GhostNearHunt,
    /// Welcome message for easy difficulty.
    MissionStartEasy,
}

impl WalkieEvent {
    fn to_concept(&self) -> Base1Concept {
        match self {
            WalkieEvent::GearInVan => Base1Concept::GearInVan,
            WalkieEvent::GhostNearHunt => Base1Concept::GhostNearHunt,
            WalkieEvent::MissionStartEasy => Base1Concept::MissionStartEasy,
        }
    }

    pub fn time_to_play(&self, count: u32) -> f64 {
        let count = count.max(1) as f64;
        match self {
            WalkieEvent::GearInVan => 120.0 * count,
            WalkieEvent::GhostNearHunt => 120.0 * count.cbrt(),
            WalkieEvent::MissionStartEasy => 3600.0 * 24.0 * 7.0,
        }
    }

    /// Get the list of voice line data for the event.
    pub fn sound_file_list(&self) -> Vec<VoiceLineData> {
        self.to_concept().get_lines()
    }
}
