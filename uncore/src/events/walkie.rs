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
    pub fn voice_text(sound_file: &str) -> &str {
        match sound_file {
            "walkie/gear-in-van-1.ogg" => {
                "Your gear is in the van. Are you sure you want to go in without it?"
            }
            "walkie/gear-in-van-2.ogg" => {
                "Uh... Aren't you forgetting something? The van is full of gear. And your hands are empty."
            }
            "walkie/gear-in-van-3.ogg" => {
                "Wait a minute, you left your kit in the truck! You might want to go back and collect your gear."
            }
            "walkie/gear-in-van-4.ogg" => {
                "Wait! Where's your equipment? You need to grab it from the van before going in."
            }
            "walkie/gear-in-van-5.ogg" => {
                "Hold on a second! Did you remember to pick up your gear from the truck?"
            }
            "walkie/gear-in-van-6.ogg" => {
                "You might want to double-check the van. It seems like you're missing some essential tools."
            }
            "walkie/gear-in-van-7.ogg" => {
                "Don't forget to equip yourself from the van before heading inside!"
            }
            _ => "**BZZT**",
        }
    }
}
