use super::ManualChapter;

pub mod page1;

pub fn create_manual_chapter() -> ManualChapter {
    ManualChapter {
        name: "Chapter 4: Expanding Your Arsenal".into(),
        pages: vec![page1::create_manual_page()],
    }
}
