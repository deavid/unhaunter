use crate::types::ManualChapter;

pub(crate) mod page1;
pub(crate) mod page2;
pub(crate) mod page3;

pub(crate) fn create_manual_chapter() -> ManualChapter {
    ManualChapter {
        pages: vec![
            page1::create_manual_page(),
            page2::create_manual_page(),
            page3::create_manual_page(),
        ],
    }
}
