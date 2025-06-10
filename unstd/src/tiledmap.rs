use std::sync::Arc;

use bevy::prelude::*;
use bevy_platform::collections::HashMap;

use crate::materials::CustomMaterial1;

#[derive(Debug, Clone)]
pub enum AtlasData {
    Sheet((Handle<TextureAtlasLayout>, CustomMaterial1)),
    Tiles(Vec<(Handle<Image>, CustomMaterial1)>),
}

#[derive(Debug, Clone)]
pub struct MapTileSet {
    pub tileset: Arc<tiled::Tileset>,
    pub data: AtlasData,
    pub y_anchor: f32,
}

#[derive(Debug, Clone, Default, Resource)]
pub struct MapTileSetDb {
    pub db: HashMap<String, MapTileSet>,
}
