//! This module defines the `WalkieEvent` enum, which represents various events
//! that can trigger walkie-talkie messages to the player.

pub mod hint;
mod walkie_config;
mod walkie_content;
pub mod walkie_types;

#[cfg(test)]
mod test_effective_priority;
