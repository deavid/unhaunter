use super::ManualChapter;

pub mod page1;
pub mod page2;
pub mod page3;

pub fn create_manual_chapter() -> ManualChapter {
    ManualChapter {
        name: "Chapter 1: Getting Started".into(),
        pages: vec![
            page1::create_manual_page(),
            page2::create_manual_page(),
            page3::create_manual_page(),
        ],
    }
}
