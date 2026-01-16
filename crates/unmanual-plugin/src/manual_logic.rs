use bevy::prelude::*;
use unmanual_core::assets::ManualAssets;
use unui_core::assets::UiAssets;

use crate::chapter1;
use crate::chapter2;
use crate::chapter3;
use crate::chapter4;
use crate::chapter5;
use crate::resources::manual::CurrentManualPage;
use crate::resources::manual::Manual;

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
    manual_assets: &ManualAssets,
    ui_assets: &UiAssets,
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
    (page.draw_fn)(parent, manual_assets, ui_assets);
}
