use bevy::prelude::*;

use unmanual_core::assets::ManualAssets;

#[derive(Debug, Clone)]
pub(crate) struct ManualPageData {
    pub draw_fn: fn(&mut ChildSpawnerCommands, &ManualAssets, &ManualAssets),
}

#[derive(Debug, Clone)]
pub(crate) struct ManualChapter {
    pub pages: Vec<ManualPageData>,
}
