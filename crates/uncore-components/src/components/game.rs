use bevy::prelude::*;

use uncore_types::types::game::SoundType;

#[derive(Component)]
pub struct GCameraArena;
#[derive(Component, Debug)]
pub struct GameSprite;

#[derive(Component, Debug)]
pub struct MapTileSprite;

#[derive(Component, Debug)]
pub struct GameSound {
    pub class: SoundType,
}
