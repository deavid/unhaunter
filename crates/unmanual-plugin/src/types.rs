use bevy::prelude::*;

use uncore_assets::types::root::game_assets::GameAssets;

#[derive(Debug, Clone)]
pub struct ManualPageData {
    pub title: String,
    pub subtitle: String,
    pub draw_fn: fn(&mut ChildSpawnerCommands, &GameAssets),
}

#[derive(Debug, Clone)]
pub struct ManualChapter {
    pub pages: Vec<ManualPageData>,
    pub name: String,
}
