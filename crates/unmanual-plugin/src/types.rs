use bevy::prelude::*;

use unmanual_core::assets::ManualAssets;
use unui_core::assets::UiAssets;

#[derive(Debug, Clone)]
pub(crate) struct ManualPageData {
    pub draw_fn: fn(&mut ChildSpawnerCommands, &ManualAssets, &UiAssets),
}

#[derive(Debug, Clone)]
pub(crate) struct ManualChapter {
    pub pages: Vec<ManualPageData>,
}
