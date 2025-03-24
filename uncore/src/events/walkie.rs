/// Part of unwalkie - a voice that plays to give tips to the player at the right time.
use bevy::prelude::*;

/// Sending this event will cause the walkie to play a message.
#[derive(Clone, Debug, Event, PartialEq, Eq, Hash)]
pub enum WalkieEvent {
    /// When the player forgets the stuff in the van.
    GearInVan,
}

impl WalkieEvent {
    /// Get the sound file for the event.
    pub fn sound_file_list(&self) -> Vec<&str> {
        match self {
            WalkieEvent::GearInVan => vec![
                "walkie/gear-in-van-1.ogg",
                "walkie/gear-in-van-2.ogg",
                "walkie/gear-in-van-3.ogg",
                "walkie/gear-in-van-4.ogg",
                "walkie/gear-in-van-5.ogg",
                "walkie/gear-in-van-6.ogg",
                "walkie/gear-in-van-7.ogg",
            ],
        }
    }
}
