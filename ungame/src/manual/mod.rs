pub mod preplay_manual_ui;
pub mod user_manual_ui;
pub mod utils;

pub mod chapter1;
pub mod chapter2;
pub mod chapter3;
pub mod chapter4;
pub mod chapter5;

use bevy::prelude::*;
pub use preplay_manual_ui::preplay_manual_system;

use crate::root::GameAssets;

pub use uncore::resources::manual::CurrentManualPage;
pub use uncore::resources::manual::Manual;
pub use uncore::types::manual::{ManualChapter, ManualPageData};

pub fn create_manual() -> Manual {
    Manual {
        chapters: vec![
            chapter1::create_manual_chapter(),
            chapter2::create_manual_chapter(),
            chapter3::create_manual_chapter(),
            chapter4::create_manual_chapter(),
            chapter5::create_manual_chapter(),
        ],
    }
}

pub fn draw_manual_page(
    parent: &mut ChildBuilder,
    handles: &GameAssets,
    manual: &Manual,
    current_page: &CurrentManualPage,
) {
    let mut chapter_index = current_page.0;
    let mut page_index = current_page.1;

    // --- Chapter Bounds Check ---
    let chapter_count = manual.chapters.len();
    if chapter_index >= chapter_count {
        warn!(
            "Chapter index out of bounds: {} (max: {})",
            chapter_index,
            chapter_count - 1
        );
        chapter_index = chapter_count - 1;
    }
    let chapter = &manual.chapters[chapter_index];

    // --- Page Bounds Check ---
    let page_count = chapter.pages.len();
    if page_index >= page_count {
        warn!(
            "Page index out of bounds: {} (max: {})",
            page_index,
            page_count - 1
        );
        page_index = page_count - 1;
    }
    let page = &chapter.pages[page_index];

    // --- Draw the Page ---
    (page.draw_fn)(parent, handles);
}

pub fn app_setup(app: &mut App) {
    user_manual_ui::app_setup(app);
    preplay_manual_ui::app_setup(app);

    app.insert_resource(create_manual());
}
