pub mod plugin;
pub(crate) mod preplay_manual_ui;
pub(crate) mod resources;
pub(crate) mod types;
pub(crate) mod user_manual_ui;
pub(crate) mod utils;

pub(crate) mod chapter1;
pub(crate) mod chapter2;
pub(crate) mod chapter3;
pub(crate) mod chapter4;
pub(crate) mod chapter5;

use bevy::prelude::*;

use uncore_assets::types::root::game_assets::GameAssets;

pub(crate) use resources::manual::CurrentManualPage;
pub(crate) use resources::manual::Manual;
pub(crate) use types::{ManualChapter, ManualPageData};

pub(crate) fn create_manual() -> Manual {
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

pub(crate) fn draw_manual_page(
    parent: &mut ChildSpawnerCommands,
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
