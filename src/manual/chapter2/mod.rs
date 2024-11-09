pub mod page1;
pub mod page2;

use crate::manual::ManualChapter;

pub fn create_manual_chapter() -> ManualChapter {
    ManualChapter {
        name: "Chapter 2: Advanced Techniques".into(),
        pages: vec![page1::create_manual_page(), page2::create_manual_page()],
    }
}
