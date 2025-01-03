pub mod page1;

use super::ManualChapter;

pub fn create_manual_chapter() -> ManualChapter {
    ManualChapter {
        name: "Chapter 3: Advanced Investigation".into(),
        pages: vec![page1::create_manual_page()],
    }
}
