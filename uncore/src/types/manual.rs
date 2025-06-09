use bevy::prelude::*;
// Attempting to use the prelude version as suggested by compiler
// If ChildSpawnerCommands is indeed in prelude for 0.16 (e.g. as an alias)
// use bevy::prelude::ChildSpawnerCommands; // This line would be redundant if it's already in bevy::prelude::*;
// No specific use needed if relying on `bevy::prelude::*` and ChildSpawnerCommands is in there.
// The signature will just be `fn(&mut ChildSpawnerCommands, ...)`.
// The previous script already made the signature `ChildSpawnerCommands`.
// The main change is removing the incorrect specific `use` statement.
// If `ChildSpawnerCommands` is NOT in `bevy::prelude::*`, this will fail,
// and we'd need to use `bevy::ecs::system::EntityCommands` or `bevy::hierarchy::ChildBuilder`.

// For now, let's assume the compiler hint about `bevy::prelude::ChildSpawnerCommands` implies it can be found via the glob import.
// So, the main action is to remove the incorrect specific import.
// The previous `perl` script also ensured the fn signature is `fn(&mut ChildSpawnerCommands, ...)`.
// Let's ensure the `use bevy::ecs::system::ChildSpawnerCommands;` is removed.
// The file content shows `use bevy::ecs::system::ChildSpawnerCommands;` and `use bevy::prelude::*;`.
// I will remove the specific, incorrect one.
use serde::{Deserialize, Serialize};

use crate::resources::manual::Manual;

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

impl ManualChapter {
    pub fn index(&self, manuals: &Manual) -> usize {
        //Find the index of `self` in manuals.chapters
        manuals
            .chapters
            .iter()
            .position(|chapter| chapter.name == self.name)
            .unwrap_or_else(|| {
                //Panic if chapter not found in manuals.chapters. This is important to detect invalid states.
                panic!("Chapter {:?} not found in manual", self.name);
            })
    }
}

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
