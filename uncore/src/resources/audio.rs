use crate::types::game::SoundType;
use bevy::prelude::*;
use bevy_platform::collections::HashMap;

/// Resource to track ambient sound instances for bevy_kira_audio
#[derive(Resource, Default)]
pub struct AmbientSoundInstances {
    pub instances: HashMap<SoundType, Handle<bevy_kira_audio::AudioInstance>>,
}
