use bevy::prelude::*;

use crate::types::game::SoundType;

pub mod ui;

#[derive(Component)]
pub struct GCameraArena;
#[derive(Component, Debug)]
pub struct GameSprite;

#[derive(Component, Debug, Default)]
pub struct MapUpdate {
    pub last_update: f32,
}

#[derive(Component, Debug)]
pub struct GameSound {
    pub class: SoundType,
}
