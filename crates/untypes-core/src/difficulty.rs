//! Basic Difficulty enum definition
//!
//! This module contains just the enum and basic utility methods.

use enum_iterator::{Sequence, all};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// Represents the different difficulty levels for the Unhaunter game.
#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    Sequence,
    Serialize,
    Deserialize,
    Default,
    Display,
    EnumString,
)]
#[strum(serialize_all = "kebab-case")]
pub enum Difficulty {
    #[default]
    TutorialChapter1,
    TutorialChapter2,
    TutorialChapter3,
    TutorialChapter4,
    TutorialChapter5,

    StandardChallenge,
    HardChallenge,
    ExpertChallenge,
    MasterChallenge,
}

impl Difficulty {
    /// Returns an iterator over all difficulty levels.
    pub fn all() -> impl Iterator<Item = Difficulty> {
        all().filter(|x: &Difficulty| x.is_enabled())
    }

    /// Returns the next difficulty level, wrapping around to the beginning if at the end.
    pub fn next(&self) -> Self {
        enum_iterator::next_cycle(self)
    }

    /// Returns the previous difficulty level, wrapping around to the end if at the beginning.
    pub fn prev(&self) -> Self {
        enum_iterator::previous_cycle(self)
    }

    pub fn is_enabled(&self) -> bool {
        matches!(
            self,
            Difficulty::TutorialChapter1
                | Difficulty::TutorialChapter2
                | Difficulty::TutorialChapter3
                | Difficulty::TutorialChapter4
                | Difficulty::TutorialChapter5
                | Difficulty::StandardChallenge
                | Difficulty::HardChallenge
                | Difficulty::ExpertChallenge
                | Difficulty::MasterChallenge
        )
    }

    pub fn is_tutorial_difficulty(&self) -> bool {
        matches!(
            self,
            Difficulty::TutorialChapter1
                | Difficulty::TutorialChapter2
                | Difficulty::TutorialChapter3
                | Difficulty::TutorialChapter4
                | Difficulty::TutorialChapter5
        )
    }
}
