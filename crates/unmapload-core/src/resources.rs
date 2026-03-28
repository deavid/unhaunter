use bevy::prelude::Resource;
use std::collections::HashMap;
use unbehavior_core::behavior::SpriteCVOKey;

use crate::components::MapTileComponents;

/// Map tile behavior index used by map loading and interaction systems.
#[derive(Clone, Default, Resource)]
pub struct SpriteDB {
    pub map_tile: HashMap<(String, u32), MapTileComponents>,
    pub cvo_idx: HashMap<SpriteCVOKey, Vec<(String, u32)>>,
}

impl SpriteDB {
    pub fn clear(&mut self) {
        self.map_tile.clear();
        self.cvo_idx.clear();
    }
}
