use bevy::prelude::*;

use crate::types::manual::ManualChapter;

#[derive(Resource, Debug, Clone)]
pub struct Manual {
    pub chapters: Vec<ManualChapter>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource, Default)]
pub struct CurrentManualPage(pub usize, pub usize); // Chapter index, Page Index
