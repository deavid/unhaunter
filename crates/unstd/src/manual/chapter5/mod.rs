use super::ManualChapter;

pub mod page1;

pub fn create_manual_chapter() -> ManualChapter {
    ManualChapter {
        name: "Chapter 5: Tools of the Adept".into(),
        pages: vec![page1::create_manual_page()],
    }
}
