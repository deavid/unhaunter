use bevy::prelude::*;

use unassets_core::types::root::game_assets::GameAssets;

#[derive(Debug, Clone)]
pub struct ManualPageData {
    pub draw_fn: fn(&mut ChildSpawnerCommands, &GameAssets),
}

#[derive(Debug, Clone)]
pub struct ManualChapter {
    pub pages: Vec<ManualPageData>,
}
