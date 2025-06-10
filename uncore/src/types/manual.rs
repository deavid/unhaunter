use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::resources::manual::Manual;

use super::root::game_assets::GameAssets;

#[derive(Debug, Clone)]
pub struct ManualPageData {
    pub title: String,
    pub subtitle: String,
    pub draw_fn: fn(&mut ChildBuilder, &GameAssets),
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
