use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::root::game_assets::GameAssets;

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

// Note: The `index` method that depended on Manual from uncore-resources
// has been moved to uncore-resources to avoid circular dependency

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ManualChapterIndex {
    Chapter1,
    Chapter2,
    Chapter3,
    Chapter4,
    Chapter5,
}

impl ManualChapterIndex {
    pub fn index(&self) -> usize {
        *self as usize
    }
}
