use bevy::prelude::*;

use crate::types::ManualChapter;

#[derive(Resource, Debug, Clone)]
pub(crate) struct Manual {
    pub chapters: Vec<ManualChapter>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource, Default)]
pub(crate) struct CurrentManualPage(pub usize, pub usize); // Chapter index, Page Index
