use bevy::prelude::*;
use unfoundation_core::types::sound::SoundType;

#[derive(Component, Debug)]
pub struct GameSprite;

#[derive(Component, Debug)]
pub struct MapTileSprite;

#[derive(Component, Debug)]
pub struct GameSound {
    pub class: SoundType,
}
