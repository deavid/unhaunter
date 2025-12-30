use bevy::prelude::*;
use uncore_foundation::types::sound::SoundType;

#[derive(Component, Debug)]
pub struct GameSprite;

#[derive(Component, Debug)]
pub struct MapTileSprite;

#[derive(Component, Debug)]
pub struct GameSound {
    pub class: SoundType,
}
