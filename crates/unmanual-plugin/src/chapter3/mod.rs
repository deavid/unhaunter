pub mod page1;

use super::ManualChapter;

pub fn create_manual_chapter() -> ManualChapter {
    ManualChapter {
        pages: vec![page1::create_manual_page()],
    }
}
