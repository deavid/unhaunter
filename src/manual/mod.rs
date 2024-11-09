pub mod chapter1;
pub mod chapter2;
pub mod preplay_manual_ui;
pub mod user_manual_ui;
pub mod utils;

use bevy::prelude::*;
use enum_iterator::Sequence;
pub use preplay_manual_ui::preplay_manual_system;

use crate::root::GameAssets;

// TODO: Remove ManualPageObsolete
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence, Resource, Default)]
pub enum ManualPageObsolete {
    #[default]
    MissionBriefing,
    EssentialControls,
    EMFAndThermometer,
    TruckJournal,
    ExpellingGhost,
}

#[derive(Debug, Clone)]
pub struct ManualPageData {
    pub title: String,
    pub subtitle: String,
    pub draw_fn: fn(&mut ChildBuilder, &GameAssets),
}

#[derive(Resource, Debug, Clone)]
pub struct Manual {
    pub chapters: Vec<ManualChapter>,
}

#[derive(Debug, Clone)]
pub struct ManualChapter {
    pub pages: Vec<ManualPageData>,
    pub name: String,
}

pub fn create_manual() -> Manual {
    Manual {
        chapters: vec![
            chapter1::create_manual_chapter(),
            chapter2::create_manual_chapter(),
        ],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource, Default)]
pub struct CurrentManualPage(pub usize, pub usize); // Chapter index, Page Index

//Generic Manual page drawing function
pub fn draw_manual_page(
    parent: &mut ChildBuilder,
    handles: &GameAssets,
    manual: &Manual,
    current_page: &CurrentManualPage,
) {
    let page_data = &manual.chapters[current_page.0].pages[current_page.1];
    (page_data.draw_fn)(parent, handles);
}

// Update ManualPage enum and its methods (see next step)

pub fn app_setup(app: &mut App) {
    user_manual_ui::app_setup(app);
    preplay_manual_ui::app_setup(app);

    app.insert_resource(create_manual());
}
