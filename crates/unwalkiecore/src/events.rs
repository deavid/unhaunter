//! This module defines the `WalkieEvent` enum, which represents various events
//! that can trigger walkie-talkie messages to the player.

mod walkie_config;
mod walkie_content;
mod walkie_types;

#[cfg(test)]
mod test_effective_priority;

pub use walkie_types::{
    WalkieEvent, WalkieEventPriority, WalkieRepeatBehavior, WalkieTalkingEvent,
};
