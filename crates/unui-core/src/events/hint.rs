use bevy::prelude::*;

/// Event that is sent when an on-screen hint should be displayed to the player
/// This is typically triggered by a walkie-talkie message
#[derive(Clone, Debug, Message)]
pub struct OnScreenHintEvent {
    /// The hint text to display to the player
    pub hint_text: String,
}