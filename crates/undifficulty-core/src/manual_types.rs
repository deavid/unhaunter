use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Reflect)]
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
