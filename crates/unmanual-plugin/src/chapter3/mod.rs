pub mod page1;

use crate::types::ManualChapter;

pub fn create_manual_chapter() -> ManualChapter {
    ManualChapter {
        pages: vec![page1::create_manual_page()],
    }
}
