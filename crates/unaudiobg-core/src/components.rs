use bevy::prelude::*;

/// Enum of background audio track types.
#[derive(Debug, PartialEq, Eq)]
pub enum SoundType {
    /// Interior house ambient
    BackgroundHouse,
    /// Exterior street ambient
    BackgroundStreet,
    /// Player heartbeat (health indicator)
    HeartBeat,
    /// Insanity effect track
    Insane,
}

/// Component marking an entity as playing a background audio track.
#[derive(Component, Debug)]
pub struct GameSound {
    pub class: SoundType,
}
