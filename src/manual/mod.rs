pub mod chapter1;
pub mod preplay_manual_ui;
pub mod user_manual_ui;
pub mod utils;

use bevy::prelude::*;
use enum_iterator::Sequence;
pub use preplay_manual_ui::preplay_manual_system;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualChapter {
    Chapter1,
    // Chapter2, // Add more chapters as needed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence, Resource, Default)]
pub enum ManualPage {
    #[default]
    MissionBriefing,
    EssentialControls,
    EMFAndThermometer,
    TruckJournal,
    ExpellingGhost,
}

pub fn app_setup(app: &mut App) {
    user_manual_ui::app_setup(app);
    preplay_manual_ui::app_setup(app);
}
