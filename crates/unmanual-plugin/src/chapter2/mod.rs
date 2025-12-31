pub mod page1;
pub mod page2;

use crate::types::ManualChapter;

pub fn create_manual_chapter() -> ManualChapter {
    ManualChapter {
        pages: vec![page1::create_manual_page(), page2::create_manual_page()],
    }
}
