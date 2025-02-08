use bevy::prelude::*;

use super::{anchors::Anchors, font_assets::FontAssets, image_assets::ImageAssets};

#[derive(Debug, Clone, Resource)]
pub struct GameAssets {
    pub images: ImageAssets,
    pub fonts: FontAssets,
    pub anchors: Anchors,
}
